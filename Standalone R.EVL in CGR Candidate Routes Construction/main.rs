//! test main for my_r_evl.rs on R.EVL of Candidate Routes Construction.
mod my_r_evl;
use my_r_evl::{Contact, compute_route_evl};

fn main() {
    let contact1 = Contact { id: 1, start: 0.0, end: 10.0, data_rate: 1.0, max_volume: 10.0 };
    let contact2 = Contact { id: 2, start: 10.0, end: 20.0, data_rate: 1.0, max_volume: 20.0 };
    let contact3 = Contact { id: 3, start: 20.0, end: 25.0, data_rate: 2.0, max_volume: 10.0 };
    let contact4 = Contact { id: 4, start: 25.0, end: 30.0, data_rate: 1.0, max_volume: 20.0 };

    // let route = vec![contact1.clone(), contact2.clone()];
    // let evl1 = contact1.max_volume.min(contact1.data_rate * (contact1.end - contact1.start));
    // let evl2 = contact2.max_volume.min(contact2.data_rate * (contact2.end - contact2.start));
    // println!("Contact {} EVL: {}", contact1.id, evl1);
    // println!("Contact {} EVL: {}", contact2.id, evl2);

    // let route_evl = compute_route_evl(&route);
    // println!("Route EVL = {}", route_evl);

    let bundle_evc = 10.0;
    println!("Bundle EVC = {}", bundle_evc);

    // if route_evl >= bundle_evc {
    //     println!("Route can forward the bundle.");
    // } else {
    //     println!("Route cannot forward the bundle (EVL < B.EVC).");
    // }

    let paths = vec![
        vec![&contact1, &contact2],
        vec![&contact1, &contact2, &contact3],
        vec![&contact2, &contact3],
        vec![&contact3, &contact4],
        vec![&contact1, &contact2, &contact3, &contact4],
    ];

    for (i, route) in paths.iter().enumerate() {
        println!("\nPath {}:", i + 1);
        for c in route {
            let evl = c.max_volume.min(c.data_rate * (c.end - c.start));
            println!("  Contact {} EVL: {}", c.id, evl);
        }

        let route_evl = compute_route_evl(route);
        println!("  Route EVL = {}", route_evl);
        // println!("  Bundle EVC = {}", bundle_evc);
        if route_evl >= bundle_evc {
            println!("Route can forward the bundle.");
        } else {
            println!("Route cannot forward the bundle (EVL < EVC).");
        }
    }
}
