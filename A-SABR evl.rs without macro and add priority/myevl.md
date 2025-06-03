# Modification needed in A-SABR crate

## To integrate myevl.rs
* In `mod.rs`, `from_ion_file.rs`, `from_tvgutil_file.rs`:
```
// pub mod evl;
pub mod myevl;
```
## Compare with macro generated evl.rs
* Use shorter type declaration. For example, from "crate::types::Bundle" to "Bundle".
* Check bundle expiration in dry_run_rx
```
        if tx_end > contact_data.end { 
            return None; // needed in algo 5 part 1, TODO: verify this.
        }
```
* Compute C.EVL in dry_run_rx and check 
```
        let max_volume = (tx_end - tx_start) * self.rate;
        if bundle.size > max_volume.min(self.get_mav(bundle.priority)) {
            return None;
        }
```
* Remove if for $add_delay:tt, $auto_update:tt = true, false.
* Parse MAV (mav_state).