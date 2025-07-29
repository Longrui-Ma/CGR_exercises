#![allow(warnings)]
use a_sabr::contact;
// RUST_BACKTRACE=1 cargo run --features "contact_work_area,first_depleted" ./02_ptvg_80_60950_3d.json
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    cell::RefCell,
    env,
    fs::File,
    io::Write,
    rc::Rc,
    time::{Duration, Instant, SystemTime},
};

use a_sabr::{
    bundle::Bundle,
    contact_manager::{
        legacy::evl::EVLManager, legacy::qd::QDManager, seg::SegmentationManager, ContactManager,
    },
    contact_plan::from_tvgutil_file::TVGUtilContactPlan,
    node_manager::none::NoManagement,
    route_storage::{cache::TreeCache, table::RoutingTable},
    routing::{
        aliases::{build_generic_router, SpsnOptions},
        Router,
    },
};

// Returns the current UTC time in HH:MM:SS format
fn time_now() -> String {
    let sec_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH!")
        .as_secs_f64();
    let sec_of_day = sec_since_epoch % 86_400.0; // 24 * 60 * 60
    let hours = (sec_of_day / 3600.0) as u32;
    let minutes = ((sec_of_day % 3600.0) / 60.0) as u32;
    let seconds = (sec_of_day % 60.0) as u32;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn percentile(sorted: &Vec<f64>, p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    let rank = p / 100.0 * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let weight = rank - lo as f64;
        sorted[lo] * (1.0 - weight) + sorted[hi] * weight
    }
}

fn run_time<NM, CM>(
    router: &mut dyn Router<NM, CM>,
    bundle: &Bundle,
    start_time: f64,
) -> (Duration, bool)
where
    NM: a_sabr::node_manager::NodeManager + 'static,
    CM: a_sabr::contact_manager::ContactManager + 'static,
{
    let start = Instant::now();
    let route_result = router.route(bundle.source, bundle, start_time, &Vec::new());
    let elapsed = start.elapsed();
    let is_success = route_result.is_some();
    (elapsed, is_success)
}

fn batch_compute_times<NM, CM>(
    routers: &mut [Box<dyn Router<NM, CM>>],
    node_count: u16,
    bundle_max_count: usize,
    bundle_min_size: f64,
    bundle_max_size: f64,
    start_time: f64,
    end_time: f64,
    throttle_on: bool,
    elapse_cap: Duration,
) -> (Vec<Vec<Duration>>, Vec<f64>, Vec<f64>)
where
    NM: a_sabr::node_manager::NodeManager + 'static,
    CM: a_sabr::contact_manager::ContactManager + 'static,
{
    // create Vec<Duration> for each router to track routing durations
    let mut durations = vec![Vec::new(); routers.len()];
    let mut bundle_schedule_rate: Vec<f64> = Vec::with_capacity(routers.len());
    let mut failure_rate: Vec<f64> = Vec::with_capacity(routers.len());
    for (router_idx, router_box) in routers.iter_mut().enumerate() {
        let router_ref: &mut dyn Router<NM, CM> = router_box.as_mut();
        let mut elapse: Duration = Duration::new(0, 0);
        let mut failure_count = 0.0;
        let mut bundle_count = bundle_max_count;
        for i in 0..bundle_max_count {
            if elapse > elapse_cap && throttle_on {
                bundle_count = i;
                break;
            }
            let mut rng = StdRng::seed_from_u64((i + 1) as u64);
            let size = rng.random_range(bundle_min_size..=bundle_max_size);
            // pick randomly the source and the destination
            let src = rng.random_range(0..node_count);
            let mut dst = rng.random_range(0..node_count);
            while dst == src {
                dst = rng.random_range(0..node_count);
            }
            let bundle = Bundle {
                source: src,
                destinations: vec![dst], // unicast
                priority: 1,
                size,
                expiration: end_time,
            };
            let (d, is_success) = run_time(router_ref, &bundle, start_time);
            elapse += d;
            if !is_success {
                failure_count += 1.0;
            };
            durations[router_idx].push(d);
        }
        bundle_schedule_rate.push((bundle_count as f64 + 1.0) / (elapse.as_nanos() as f64) / 1e-9); // (i+1) as f64 / durations[router_idx].iter().sum::<Duration>().as_nanos() as f64 / 1e-9,
        failure_rate.push(failure_count / (bundle_count as f64 + 1.0));
    }
    (durations, bundle_schedule_rate, failure_rate)
}

/// CM（EVL, QD, Seg）measure and print
fn measure_for<CM>(
    cm_label: &str,
    router_configs: &Vec<(&'static str, Option<SpsnOptions>)>,
    router_names: &Vec<&'static str>,
    cp_file: &str,
    node_count: u16,
    contact_count: usize,
    bundle_max_count: usize,
    bundle_min_size: f64,
    bundle_max_size: f64,
    start_time: f64,
    end_time: f64,
    elapse_cap: Duration,
    throttle_on: bool,
    export_csv: bool,
    tvgutil_seed: u64,
) where
    CM: ContactManager + 'static,
{
    let mut routers_box: Vec<Box<dyn Router<NoManagement, EVLManager>>> = Vec::new();
    for (name, options) in router_configs.into_iter() {
        let (nodes, contacts) = TVGUtilContactPlan::parse::<NoManagement, EVLManager>(cp_file)
            .expect("!!!Failed to parse contact plan");
        let router = build_generic_router(name, nodes, contacts, options.clone());
        routers_box.push(router);
    }

    println!(
        "{}, Measuring compute time for all routers with {}",
        time_now(),
        cm_label
    );
    let (compute_times, schedule_rate, failure_rate) = batch_compute_times(
        &mut routers_box,
        node_count,
        bundle_max_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
        end_time,
        throttle_on,
        elapse_cap,
    );
    // println!(
    //     "{}, Finished measuring compute times for all routers with {}.",
    //     time_now(),
    //     cm_label
    // );
    // export CSV: metrics rows, algos columns like print
    let mut file: Option<File> = None;
    if export_csv {
        let filename = format!(
            "../results/{}_{}_{}_1e{}b_{}s_{}.csv",
            node_count,
            contact_count,
            cm_label,
            (bundle_max_count as f64).log10().round() as usize,
            elapse_cap.as_secs(),
            tvgutil_seed
        );
        let f = File::create(&filename).expect("!Can not create CSV file");
        // header
        writeln!(
            &f,
            "algo,mean_ns,std_ns,fail_rate,sch_rate,sum_s,p0,p5,p10,p20,p50,p80,p90,p95,p100"
        )
        .unwrap();
        file = Some(f); // move into file
    }
    for (i, name) in router_names.iter().enumerate() {
        let times = &compute_times[i];
        let sum_ns = times.iter().sum::<Duration>().as_nanos() as f64; // nanoseconds
        let sum_s = sum_ns / 1e9; // seconds
        let mut ns_vals: Vec<f64> = times.iter().map(|d| d.as_nanos() as f64).collect();
        ns_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean_ns = sum_ns / ns_vals.len() as f64;
        let var_ns =
            ns_vals.iter().map(|&v| (v - mean_ns).powi(2)).sum::<f64>() / ns_vals.len() as f64;
        let std_ns = var_ns.sqrt();
        let p0 = percentile(&ns_vals, 0.0); // ns
        let p5 = percentile(&ns_vals, 5.0);
        let p10 = percentile(&ns_vals, 10.0);
        let p20 = percentile(&ns_vals, 20.0);
        let p50 = percentile(&ns_vals, 50.0);
        let p80 = percentile(&ns_vals, 80.0);
        let p90 = percentile(&ns_vals, 90.0);
        let p95 = percentile(&ns_vals, 95.0);
        let p100 = percentile(&ns_vals, 100.0);
        let sch_rate = schedule_rate[i];
        let fail_rate = failure_rate[i];
        println!(
            "{:32}:mean= {:>8.0} ns,std= {:>8.0} ns,failure rate= {:>6.2}%,schedule rate= {:>9.2} bundles/sec,total time= {:>4.2} s,min= {:>6.0} ns,max= {:>6.2} ms.",
            name,
            mean_ns,
            std_ns,
            fail_rate * 100.0,
            sch_rate,
            sum_s,
            p0,
            p100 / 1e6,
        );
        if let Some(f) = file.as_mut() {
            writeln!(
                f,
                "{},{:.0},{:.0},{:.4},{:.2},{:.2},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0}",
                name, mean_ns, std_ns, fail_rate, sch_rate, sum_s,
                p0, p5, p10, p20, p50, p80, p90, p95, p100,
            )
            .unwrap();
        }
    }
    // println!("\n{}, Finished compute stats with {}.\n", time_now(), cm_label);
}

fn main() {
    // manual input parameters
    let data_rate = 9600.0; // field `rate` in `ContactManager` is private
    let bundle_max_count = 1e5 as usize; // max number of bundles that a router will route
    let bundle_size_min_ratio = 0.01;
    let bundle_size_max_ratio = 0.1;
    let elapse_cap: Duration = Duration::from_secs(4);
    let throttle_on = true;
    let export_csv = false;
    // parse from file and get contact plan statistics
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <cp_file> <seed for tvgutil>", args[0]);
        std::process::exit(1);
    }
    let cp_file = &args[1];
    let tvgutil_seed = args[2].parse::<u64>().unwrap_or_else(|_| {
        eprintln!("ERR: Invalid seed value: {}", args[2]);
        std::process::exit(1);
    });
    let (nodes_stat, contacts_stat) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(cp_file)
            .expect("!!!Failed to parse contact plan");
    let node_count = nodes_stat.len() as u16;
    let mut earliest_date = f64::MAX; // earliest contact start time to have bundles without expiration
    let mut latest_date = 0.0; // latest contact end time to have bundles without expiration
    let mut total_volume = 0.0;
    let mut contact_count = 0;
    for contact in contacts_stat.iter() {
        if contact.info.start < earliest_date {
            earliest_date = contact.info.start;
        }
        if contact.info.end > latest_date {
            latest_date = contact.info.end;
        }
        let duration = contact.info.end - contact.info.start;
        // data_rate = contact.manager.rate; // field `rate` in `ContactManager` is private
        total_volume += duration;
        contact_count += 1;
    }
    let avg_volume = total_volume * data_rate / contact_count as f64;
    let bundle_min_size = avg_volume * bundle_size_min_ratio;
    let bundle_max_size = avg_volume * bundle_size_max_ratio;
    println!("{}, Working with cp {}, \n    which contains {} nodes, {} contacts with an average {:.2} contact volume, \n    first contact at {}, last contact at {}.", time_now(), cp_file, node_count, contact_count, avg_volume, earliest_date, latest_date);
    // generate routers
    let spsn_options = Some(SpsnOptions {
        check_size: true,
        check_priority: false,
        max_entries: 10,
    });
    let router_configs = vec![
        ("SpsnHybridParenting", spsn_options.clone()),
        ("SpsnNodeParenting", spsn_options.clone()),
        ("SpsnContactParenting", spsn_options.clone()),
        ("VolCgrHybridParenting", spsn_options.clone()),
        ("VolCgrNodeParenting", spsn_options.clone()),
        ("VolCgrContactParenting", spsn_options.clone()),
        // ("CgrFirstEndingHybridParenting", None),
        // ("CgrFirstEndingNodeParenting", None),
        // ("CgrFirstEndingContactParenting", None),
        // ("CgrFirstDepletedHybridParenting", None),
        // ("CgrFirstDepletedNodeParenting", None),
        // ("CgrFirstDepletedContactParenting", None),
    ];
    let router_names: Vec<&str> = router_configs.iter().map(|&(name, _)| name).collect();
    measure_for::<EVLManager>(
        "EVLManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        contact_count,
        bundle_max_count,
        bundle_min_size,
        bundle_max_size,
        earliest_date,
        latest_date,
        elapse_cap,
        throttle_on,
        export_csv,
        tvgutil_seed,
    );
    measure_for::<QDManager>(
        "QDManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        contact_count,
        bundle_max_count,
        bundle_min_size,
        bundle_max_size,
        earliest_date,
        latest_date,
        elapse_cap,
        throttle_on,
        export_csv,
        tvgutil_seed,
    );
    measure_for::<SegmentationManager>(
        "SegmentationManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        contact_count,
        bundle_max_count,
        bundle_min_size,
        bundle_max_size,
        earliest_date,
        latest_date,
        elapse_cap,
        throttle_on,
        export_csv,
        tvgutil_seed,
    );
}
