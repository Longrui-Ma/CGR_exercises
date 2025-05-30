# Modification needed in A-SABR crate

## To integrate myevl.rs
* In `mod.rs`:
```
// pub mod evl;
pub mod myevl;
```
In `pub trait ContactManager`
```
    // fn try_init(&mut self, contact_data: &ContactInfo) -> bool;
    fn try_init(&mut self, contact_data: &ContactInfo, bundle: Option<&Bundle>) -> bool;
```
TODO： modification needed in macro:
```
handler.try_init(&contact, Some(&bundle));
handler.try_init(&contact, None);
```
* TODO: In `types.rs` need modify Token trait for parsing.
* TODO: In `contact.rs` and `mod.rs`s add bundle in try_init.
## Compare with macro generated evl.rs
* Use shorter type declaration. For example, from "crate::types::Bundle" to "Bundle".
* Check bundle expiration in dry_run_rx
```
        if tx_end > contact_data.end { 
            return None; // needed in algo 5 part 1, TODO: verify this.
        }
```
* Remove if for $add_delay:tt, $auto_update:tt = true, false.
* TODO: Add new type parser