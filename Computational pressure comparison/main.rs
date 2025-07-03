#![allow(warnings)]
// RUST_BACKTRACE=1 cargo run --features "contact_work_area,contact_suppression,first_depleted" ./02_ptvg.json
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
    contact_manager::legacy::evl::EVLManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan,
    node_manager::none::NoManagement,
    route_storage::{cache::TreeCache, table::RoutingTable},
    routing::{
        aliases::{SpsnMpt, SpsnNodeGraph,SpsnContactGraph, CgrFirstEndingMpt, CgrFirstDepletedMpt, CgrFirstEndingNodeGraph, CgrFirstDepletedNodeGraph, CgrFirstEndingContactGraph, CgrFirstDepletedContactGraph},
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
    routers: &mut [(&str, &mut dyn Router<NM, CM>)],
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
    // create Vec<Duration> for each router
    let mut durations = vec![Vec::with_capacity(bundle_count); routers.len()];
    for i in 0..bundle_count {
        let mut rng = StdRng::seed_from_u64((i + 1) as u64);
        // let size = rng.random_range(bundle_min_size..=bundle_max_size) as f64;
        // pick randomly the source and the destination
        // let src = rng.random_range(0..node_count);
        // let mut dst = rng.random_range(0..node_count);
        // while dst == src {
        //     dst = rng.random_range(0..node_count);
        // }
        let size = 40.0;
        let src = 0;
        let dst = 1;
        let bundle = Bundle {
            source: src,
            destinations: vec![dst], // unicast
            priority: 1,
            size,
            expiration: 10_000.0,
        };
        println!(
            "......Routing bundle {}: src = {}, dst = {}, size = {}",
            i, bundle.source, bundle.destinations[0], bundle.size
        );
        for (j, (_name, router)) in routers.iter_mut().enumerate() {
            println!(
                "{}, Routing bundle {} with router {}...",
                time_now(),
                i,
                _name
            );
            let d = run_time(*router, &bundle, start_time);
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cp_file>", args[0]);
        std::process::exit(1);
    }

    println!("{}, Working with cp {}.", time_now(), args[1]);

    let start_time = 1751402805.0; // 2024-06-01T00:00:05Z
    // assume no fragmentation
    let node_count = 80;
    let bundle_count = 1;
    let bundle_min_size = 8; // 8 bytes UDP headers
    let bundle_max_size = 1_500; // 1500 bytes MTU

    // routers
    let (nodes_spsn_mpt, contacts_spsn_mpt) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let spsn_mpt_table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    let mut spsn_mpt =
        SpsnMpt::<NoManagement, EVLManager>::new(nodes_spsn_mpt, contacts_spsn_mpt, spsn_mpt_table, false);

    let (nodes_spsn_ng, contacts_spsn_ng) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let spsn_ng_table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    let mut spsn_ng =
        SpsnNodeGraph::<NoManagement, EVLManager>::new(nodes_spsn_ng, contacts_spsn_ng, spsn_ng_table, false);

    let (nodes_spsn_cg, contacts_spsn_cg) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let spsn_cg_table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    let mut spsn_cg =
        SpsnContactGraph::<NoManagement, EVLManager>::new(nodes_spsn_cg, contacts_spsn_cg, spsn_cg_table, false);

    let (nodes_cgr_fe_mpt, contacts_cgr_fe_mpt) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fe_mpt_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fe_mpt =
        CgrFirstEndingMpt::<NoManagement, EVLManager>::new(nodes_cgr_fe_mpt, contacts_cgr_fe_mpt, cgr_fe_mpt_table);

    let (nodes_cgr_fe_ng, contacts_cgr_fe_ng) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fe_ng_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fe_ng =
        CgrFirstEndingNodeGraph::<NoManagement, EVLManager>::new(nodes_cgr_fe_ng, contacts_cgr_fe_ng, cgr_fe_ng_table);

    let (nodes_cgr_fe_cg, contacts_cgr_fe_cg) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fe_cg_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fe_cg =
        CgrFirstEndingContactGraph::<NoManagement, EVLManager>::new(nodes_cgr_fe_cg, contacts_cgr_fe_cg, cgr_fe_cg_table);  

    let (nodes_cgr_fd_mpt, contacts_cgr_fd_mpt) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fd_mpt_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fd_mpt =
        CgrFirstDepletedMpt::<NoManagement, EVLManager>::new(nodes_cgr_fd_mpt, contacts_cgr_fd_mpt, cgr_fd_mpt_table);

    let (nodes_cgr_fd_ng, contacts_cgr_fd_ng) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fd_ng_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fd_ng =
        CgrFirstDepletedNodeGraph::<NoManagement, EVLManager>::new(nodes_cgr_fd_ng, contacts_cgr_fd_ng, cgr_fd_ng_table);

    let (nodes_cgr_fd_cg, contacts_cgr_fd_cg) =
        TVGUtilContactPlan::parse::<NoManagement, EVLManager>(&args[1]).unwrap();
    let cgr_fd_cg_table = Rc::new(RefCell::new(RoutingTable::new()));
    let mut cgr_fd_cg =
        CgrFirstDepletedContactGraph::<NoManagement, EVLManager>::new(nodes_cgr_fd_cg, contacts_cgr_fd_cg, cgr_fd_cg_table);

    macro_rules! entry {
        ($r:ident) => {
            (
                stringify!($r),
                &mut $r as &mut dyn Router<NoManagement, EVLManager>,
            )
        };
    }
    let mut routers: Vec<(&str, &mut dyn Router<NoManagement, EVLManager>)> =
        vec![entry!(spsn_mpt), entry!(spsn_ng), entry!(spsn_cg), entry!(cgr_fe_mpt), entry!(cgr_fe_ng), entry!(cgr_fd_mpt), entry!(cgr_fd_ng), entry!(cgr_fe_cg), entry!(cgr_fd_cg)];

    println!("{}, Measuring compute times for all routers:", time_now());
    let compute_times = batch_compute_times(
        &mut routers,
        node_count,
        bundle_count,
        bundle_min_size,
        bundle_max_size,
        start_time,
    );
    println!(
        "{}, Finished measuring compute times for all routers.\n",
        time_now()
    );

    for ((name, _), times) in routers.iter().zip(compute_times.iter()) {
        // println!("{}: {:?}", name, times);
        let sum: Duration = times.iter().sum();
        let avg = (sum / (times.len() as u32)).as_millis();
        let max = (*times.iter().max().unwrap()).as_millis();
        let min = (*times.iter().min().unwrap()).as_millis();

        println!(
            "{:30}: total = {:<20?}, avg = {:>8?}ms, max = {:>8?}ms, min = {:>8?}ms",
            name, sum, avg, max, min
        );
    }
    println!("\n{}, Finished compute stats.", time_now());
}
