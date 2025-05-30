//! An implementation of R.EVL in Algo.5 Candidate Routes Construction P16 CGR tutorial.
//! 2020_Routing in the space internet_A contact graph routing tutorial
/// effective_duration = effective_stop - start
/// effective_stop = min(contact.end, all following contact end)
/// C.EVL = min(contact.max_volume, data_rate * effective_duration) without priority impl for C.MAV
/// R.EVL = min(all C.EVL)
// #[derive(Debug，Clone)]
pub struct Contact {
    pub id: u32,
    pub start: f64,
    pub end: f64,
    pub data_rate: f64,
    pub max_volume: f64,
}

pub fn compute_route_evl(route: &[&Contact]) -> f64 { //&[&Contact] borrow instead of clone
    // R.EVL starts with inf
    let mut route_evl = f64::INFINITY;

    for (i, contact) in route.iter().enumerate() {
        let mut effective_stop = contact.end;
        for succ in route.iter().skip(i + 1) {  // all following contacts
            if succ.end < effective_stop {
                effective_stop = succ.end;
            }
        }
        let effective_duration = if effective_stop > contact.start {
            effective_stop - contact.start
        } else {
            0.0
        };
        let contact_evl = contact.max_volume.min(contact.data_rate * effective_duration);
        if contact_evl < route_evl {
         route_evl = contact_evl;
        }
    }

    if route.is_empty() {
        0.0
    } else {
     route_evl
    }
}
