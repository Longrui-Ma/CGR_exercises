#![allow(warnings)]
// RUST_BACKTRACE=1 cargo run --features "contact_work_area,first_depleted" ./02_ptvg_80_60950_3d.json
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::SystemTime;
use std::{
    cell::RefCell,
    env,
    rc::Rc,
    time::{Duration, Instant},
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

fn run_time<NM, CM>(router: &mut dyn Router<NM, CM>, bundle: &Bundle, start_time: f64) -> Duration
where
    NM: a_sabr::node_manager::NodeManager + 'static,
    CM: a_sabr::contact_manager::ContactManager + 'static,
{
    let start = Instant::now();
    router.route(bundle.source, bundle, start_time, &Vec::new());
    start.elapsed()
}

fn batch_compute_times<NM, CM>(
    routers: &mut [Box<dyn Router<NM, CM>>],
    node_count: u16,
    bundle_count: usize,
    bundle_min_size: i32,
    bundle_max_size: i32,
    start_time: f64,
) -> Vec<Vec<Duration>>
where
    NM: a_sabr::node_manager::NodeManager + 'static,
    CM: a_sabr::contact_manager::ContactManager + 'static,
{
    // create Vec<Duration> for each router to track routing durations
    let mut durations = vec![Vec::with_capacity(bundle_count); routers.len()];
    for i in 0..bundle_count {
        let mut rng = StdRng::seed_from_u64((i + 1) as u64);
        let size = rng.random_range(bundle_min_size..=bundle_max_size) as f64;
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
            expiration: start_time + 30_758_400.0, // never expires in 365 days
        };
        for (j, router_box) in routers.iter_mut().enumerate() {
            let router_ref: &mut dyn Router<NM, CM> = router_box.as_mut();
            let d = run_time(router_ref, &bundle, start_time);
            durations[j].push(d);
        }
        // if i != 0 && (i+1) % 100 == 0 {
        //     println!("...Routed {} bundles.", i+1);
        // }
    }
    durations
}

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

/// CM（EVL, QD, Seg）measure and print
fn measure_for<CM>(
    cm_label: &str,
    router_configs: &Vec<(&'static str, Option<SpsnOptions>)>,
    router_names: &Vec<&'static str>,
    cp_file: &str,
    node_count: u16,
    bundle_count: usize,
    bundle_min_size: i32,
    bundle_max_size: i32,
    start_time: f64,
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
    let compute_times = batch_compute_times(
        &mut routers_box,
        node_count,
        bundle_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
    );
    // println!(
    //     "{}, Finished measuring compute times for all routers with {}.\n",
    //     time_now(),
    //     cm_label
    // );
    for (i, name) in router_names.iter().enumerate() {
        // println!("{}: {:?}", name, times);
        let times = &compute_times[i];
        let sum: Duration = times.iter().sum();
        let avg = (sum / (times.len() as u32)).as_millis();
        let max = (*times.iter().max().unwrap()).as_millis();
        let min = (*times.iter().min().unwrap()).as_micros();
        println!(
            "{:30}: total = {:>8?} ms, max = {:>8?} ms, min = {:>8?} us, avg = {:>8?} ms",
            name, sum.as_millis(), max, min, avg
        );
    }
    // println!("\n{}, Finished compute stats with.", time_now(), cm_label);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <cp_file>", args[0]);
        std::process::exit(1);
    }
    let cp_file = &args[1];
    println!("{}, Working with cp {}.", time_now(), cp_file);
    let start_time = 1752098400.0; // 2025-07-10T00:00:00Z
                                   // assume no fragmentation？
    let node_count = 84;
    let bundle_count = 100;
    let bundle_min_size = 3_000;
    let bundle_max_size = 300_000;

    // generate routers
    let spsn_options = Some(SpsnOptions {
        check_size: true,
        check_priority: false,
        max_entries: 10,
    });

    let router_configs = vec![
        ("SpsnMpt", spsn_options.clone()),
        ("SpsnNodeGraph", spsn_options.clone()),
        ("SpsnContactGraph", spsn_options.clone()),
        ("VolCgrMpt", spsn_options.clone()),
        ("VolCgrNodeGraph", spsn_options.clone()),
        ("VolCgrContactGraph", spsn_options.clone()),
        ("CgrFirstEndingMpt", None),
        ("CgrFirstEndingNodeGraph", None),
        ("CgrFirstEndingContactGraph", None),
        ("CgrFirstDepletedMpt", None),
        ("CgrFirstDepletedNodeGraph", None),
        ("CgrFirstDepletedContactGraph", None),
    ];
    let router_names: Vec<&str> = router_configs.iter().map(|&(name, _)| name).collect();

    measure_for::<EVLManager>(
        "EVLManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        bundle_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
    );
    measure_for::<QDManager>(
        "QDManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        bundle_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
    );
    measure_for::<SegmentationManager>(
        "SegmentationManager",
        &router_configs,
        &router_names,
        cp_file,
        node_count,
        bundle_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
    );
}
