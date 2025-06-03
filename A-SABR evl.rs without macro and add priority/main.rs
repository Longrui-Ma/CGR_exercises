// example for testing priority functionality with EVLManager

use std::{cell::RefCell, env, rc::Rc};
use a_sabr::{
    bundle::Bundle,
    contact_manager::{eto::ETOManager, qd::QDManager, seg::SegmentationManager, ContactManager},
    contact_manager::myevl::EVLManager,
    contact_plan::{
        asabr_file_lexer::FileLexer,
        from_asabr_lexer::ASABRContactPlan,
    },
    node_manager::none::NoManagement,
    parsing::{coerce_cm, ContactDispatcher, Dispatcher},
    route_storage::cache::TreeCache,
    routing::{aliases::{SpsnMpt, CgrMpt}, Router},
    utils::pretty_print,
};

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cp_file>", args[0]);
        std::process::exit(1);
    }
    println!("Working with cp: {}", args[1]);
    
    // Create a lexer to retrieve tokens from a file
    let mut mylexer = FileLexer::new(&args[1]).unwrap();
    let mut cp = ASABRContactPlan::new();
    
    // Register our EVLManager as the handler for "evl" markers in the contact plan
    let mut contact_dispatch: Dispatcher<ContactDispatcher> = Dispatcher::<ContactDispatcher>::new();
    contact_dispatch.add("evl", coerce_cm::<EVLManager>);
    // contact_dispatch.add("qd", coerce_cm::<QDManager>);
    // contact_dispatch.add("eto", coerce_cm::<ETOManager>);
    // contact_dispatch.add("seg", coerce_cm::<SegmentationManager>);

    // arse the contact plan (A-SABR format thanks to ASABRContactPlan) and the lexer
    let (nodes, contacts) = cp
        .parse::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>(
            &mut mylexer,
            None,
            Some(&contact_dispatch)
        )
        .unwrap();
    
    println!("\nContact plan loaded with {} nodes and {} contacts", nodes.len(), contacts.len());
    
    // Create storage for the paths
    let table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    
    // Initialize the SPSN routing algorithm
    let mut spsn = SpsnMpt::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>::new(
        nodes.clone(),
        contacts.clone(),
        table.clone(),
        false
    );
    
    // Initialize the CGR routing algorithm
    let mut cgr = CgrMpt::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>::new(
        nodes.clone(),
        contacts.clone(),
        table.clone(),
        false
    );
    
    // Create bundles with different priorities
    let bundles = vec![
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 0, // Highest priority
            size: 5.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![3],
            priority: 1,
            size: 5.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![4],
            priority: 2,
            size: 5.0,
            expiration: 10000.0,
        },
    ];
    
    // Test SPSN routing with different priority bundles
    println!("\n=== TESTING SPSN ROUTING WITH PRIORITY ===");
    test_routing_with_priority(&mut spsn, &bundles, "SPSN");
    
    // Test CGR routing with different priority bundles
    println!("\n=== TESTING CGR ROUTING WITH PRIORITY ===");
    test_routing_with_priority(&mut cgr, &bundles, "CGR");
    
    // Test the effect of priority on resource depletion
    println!("\n=== TESTING PRIORITY EFFECT ON RESOURCE DEPLETION ===");
    test_priority_resource_depletion();
}

// Function to test routing with bundles of different priorities
fn test_routing_with_priority<R>(
    router: &mut R, 
    bundles: &[Bundle], 
    algorithm_name: &str
) 
where 
    R: Router<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>
{
    for (i, bundle) in bundles.iter().enumerate() {
        println!("\nRouting bundle {} with {} (priority: {}, size: {}, destination: {})", 
                 i + 1, algorithm_name, bundle.priority, bundle.size, bundle.destinations[0]);
        
        // Schedule the bundle (resource updates are conducted)
        let out = router.route(0, bundle, 0.0, &Vec::new());
        
        // Print routing results
        match out {
            Some((contact, route)) => {
                println!("  Route found:");
                println!("  First hop contact: from {} to {}, time [{}, {}]",
                         contact.from, contact.to, contact.start, contact.end);
                println!("  Route hops: {}", route.len());
                
                // Print each hop in the route
                for (j, hop) in route.iter().enumerate() {
                    println!("    Hop {}: from {} to {}, time [{}, {}]",
                             j + 1, hop.from, hop.to, hop.start, hop.end);
                }
            },
            None => {
                println!("  No route found for bundle {} (priority: {})", 
                         i + 1, bundle.priority);
            }
        }
    }
}

// Function to test how priority affects resource depletion
fn test_priority_resource_depletion() {
    // Create a new lexer and contact plan for this test
    let mut mylexer = FileLexer::new("./priority_test.cp").unwrap();
    let mut cp = ASABRContactPlan::new();
    
    // Register our EVLManager as the handler for "evl" markers
    let mut contact_dispatch: Dispatcher<ContactDispatcher> = Dispatcher::<ContactDispatcher>::new();
    contact_dispatch.add("evl", coerce_cm::<EVLManager>);
    
    // Parse the contact plan
    let (nodes, contacts) = cp
        .parse::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>(
            &mut mylexer,
            None,
            Some(&contact_dispatch)
        )
        .unwrap();
    
    // Create storage for the paths
    let table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    
    // Initialize the SPSN routing algorithm
    let mut spsn = SpsnMpt::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>::new(
        nodes,
        contacts,
        table,
        false
    );
    
    // Create a sequence of bundles to demonstrate resource depletion
    let depletion_bundles = vec![
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 2,
            size: 2.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 1,
            size: 4.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 0,
            size: 8.0,
            expiration: 10000.0,
        },
    ];
    
    println!("\nDemonstrating priority-based resource allocation:");
    
    // Route each bundle in sequence to show how priority affects resource allocation
    for (i, bundle) in depletion_bundles.iter().enumerate() {
        println!("\nRouting bundle {} (priority: {}, size: {})", 
                 i + 1, bundle.priority, bundle.size);
        
        // Route the bundle
        let out = spsn.route(0, bundle, 0.0, &Vec::new());
        
        // Print routing results
        match out {
            Some((contact, route)) => {
                println!("  Route found!");
                println!("  First hop: from {} to {}, time [{}, {}]",
                         contact.from, contact.to, contact.start, contact.end);
                
                // Print the complete route
                println!("  Complete route:");
                pretty_print(route);
            },
            None => {
                println!("  No route found - insufficient resources for this priority level");
            }
        }
    }
    
    // Demonstrate how high priority can still route even when lower priorities cannot
    println!("\nDemonstrating high priority routing after resource depletion:");
    
    // Create a fresh router with a new contact plan
    let mut mylexer = FileLexer::new("contact_plans/priority_test.cp").unwrap();
    let mut cp = ASABRContactPlan::new();
    
    let (nodes, contacts) = cp
        .parse::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>(
            &mut mylexer,
            None,
            Some(&contact_dispatch)
        )
        .unwrap();
    
    let table = Rc::new(RefCell::new(TreeCache::new(true, false, 10)));
    let mut spsn = SpsnMpt::<NoManagement, Box<dyn a_sabr::contact_manager::ContactManager>>::new(
        nodes,
        contacts,
        table,
        false
    );
    
    // First deplete lower priority resources
    let depletion_bundle = Bundle {
        source: 0,
        destinations: vec![2],
        priority: 2,  // Low priority (2)
        size: 3.0,    // Depletes the low priority resources
        expiration: 10000.0,
    };
    
    println!("\nFirst, routing a bundle to deplete low priority resources:");
    println!("Bundle: priority {}, size {}", depletion_bundle.priority, depletion_bundle.size);
    
    let out = spsn.route(0, &depletion_bundle, 0.0, &Vec::new());
    match out {
        Some(_) => println!("  Route found and resources allocated"),
        None => println!("  No route found")
    }
    
    // Now try to route bundles with different priorities
    let test_bundles = vec![
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 2,  // should fail (resources depleted)
            size: 1.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 1,  // should succeed
            size: 1.0,
            expiration: 10000.0,
        },
        Bundle {
            source: 0,
            destinations: vec![2],
            priority: 0,  // should succeed
            size: 1.0,
            expiration: 10000.0,
        },
    ];
    
    println!("\nNow testing bundles with different priorities after depletion:");
    
    for (i, bundle) in test_bundles.iter().enumerate() {
        println!("\nTesting bundle with priority: {}, size: {}", bundle.priority, bundle.size);
        
        let out = spsn.route(0, bundle, 0.0, &Vec::new());
        
        match out {
            Some(_) => println!("  SUCCESS: Route found - priority {} can still allocate resources", bundle.priority),
            None => println!("  FAILED: No route found - priority {} resources are depleted", bundle.priority)
        }
    }
}
