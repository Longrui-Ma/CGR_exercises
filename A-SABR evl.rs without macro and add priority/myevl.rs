//! Manually implemented Effective Volume Limit with priority support
// Based on the macro expansion of generate_basic_volume_manager but with priority

use crate::{
    bundle::Bundle,
    contact::ContactInfo,
    contact_manager::{ContactManager, ContactManagerTxData},
    types::{Date, DataRate, Duration, Volume, Priority},
};

/// A volume manager implementing the Effective Volume Limit (EVL) logic with priority support.
/// 
/// Compilation rules:
/// * Consider the delay to offset the earliest transmission opportunity: `false`.
/// * Update automatically the booked volume (i.e. queue) upon schedule: `true`. No enqueue or dequeue methods.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EVLManager {
    /// The data transmission rate.
    pub rate: DataRate,
    /// The delay between transmissions.
    pub delay: Duration,
    /// The volume scheduled for this contact.
    pub queue_size: Volume,
    /// The total volume at initialization.
    pub original_volume: Volume,
    /// Current Maximum Available Volumes for priorities (C.MAV(p))
    pub mav: Vec<Volume>,
}

impl EVLManager {
    /// Creates a new `EVLManager` with specified average rate, delay, and original MAV values.
    ///
    /// # Arguments
    ///
    /// * `rate` - The average data rate for this contact.
    /// * `delay` - The link delay for this contact.
    /// * `original_mav` - Vector of Maximum Available Volumes for each priority level.
    ///
    /// # Returns
    ///
    /// A new instance of `EVLManager`.
    pub fn new(rate: DataRate, delay: Duration, original_mav: Vec<Volume>) -> Self {
        Self {
            rate,
            delay,
            queue_size: 0.0,
            original_volume: 0.0,
            mav: original_mav,
        }
    }
    
    /// Get Maximum Available Volume for a given priority, 
    /// Returns the MAV value for the specified priority level.
    fn get_mav(&self, priority: Priority) -> Volume {
        let p = priority as usize;
        if p < self.mav.len() {
            self.mav[p]
        } else {
            // Return 0 if priority is out of range / not defined.
            0.0
        }
    }
    
    /// Update the MAV for a specific priority level after scheduling a bundle.
    // P18: Whenever a bundle B is enqueued for transmission via a particular route, 
    // the C.MAV(p) of all contacts in that route, for that bundle’s level of priority p 
    // and every lower level of priority, needs to be reduced by B.EVC.
    fn update_mav(&mut self, vol: Volume, priority: Priority) {
        let p = priority as usize;
        if p < self.mav.len() {
            // Deduct volume from the specified and lower prioritys' MAV.
            for i in p..self.mav.len() {
                if self.mav[i] > vol {
                    self.mav[i] -= vol;
                } else {
                    self.mav[i] = 0.0; // TODO: once got 0, lower mav can be set to 0 directly.
                }
            }
        }
    }
    
    // /// Calculate the Effective Volume Limit for a contact (C.EVL). 
    // // This implements step 4 of Algorithm 5 in CGR tutorial. TODO: intergrate in ContactManager try_init trait.
    // fn calculate_evl(
    //     &self, 
    //     contact_data: &ContactInfo,
    //     effective_start_time: Date,
    //     effective_stop_time: Date,
    //     bundle: &Bundle
    // ) -> Volume {
    //     // Calculate effective duration
    //     let effective_duration = if effective_stop_time > effective_start_time {
    //         effective_stop_time - effective_start_time
    //     } else {
    //         0.0
    //     };
        
    //     // Calculate maximum volume that can be transmitted during effective duration.
    //     let max_volume = effective_duration * contact_data.rate;

    //     // EVL is the minimum of max_volume and mav
    //     max_volume.min(self.get_mav(bundle.priority))
    // }
}

// Using the ContactManager trait originated in mod.rs to implement the methods for EVLManager.
impl ContactManager for EVLManager {
    /// Simulates the transmission of a bundle based on the contact data and available free intervals.
    ///
    /// The transmission time start time will NOT be offset by the queue size: `false`.
    ///
    /// # Arguments
    ///
    /// * `contact_data` - Reference to the contact information.
    /// * `at_time` - The current time for scheduling purposes.
    /// * `bundle` - The bundle to be transmitted.
    ///
    /// # Returns
    ///
    /// Optionally returns `ContactManagerTxData` with transmission start and end times, or `None` if the bundle can't be transmitted.
    fn dry_run_tx(
        &self,
        contact_data: &ContactInfo,
        at_time: Date,
        bundle: &Bundle,
    ) -> Option<ContactManagerTxData> {
        // Check if there's enough volume available for this priority
        if bundle.size > self.get_mav(bundle.priority) {
            return None;
        }
        
        // Determine the effective start and effective end time.
        let tx_start = if contact_data.start > at_time {
            contact_data.start
        } else {
            at_time
        };
        let tx_end = tx_start + bundle.size / self.rate;
        
        // Check if transmission would end after contact end
        if tx_end > contact_data.end { 
            return None; // needed in algo 5 part 1, TODO: verify this.
        }

        // Check if arrival time is after bundle expiration
        let arrival = self.delay + tx_end;
        if arrival > bundle.expiration {
            return None;
        }
        
        // Return transmission data
        Some(ContactManagerTxData {
            tx_start,
            tx_end,
            delay: self.delay,
            expiration: contact_data.end,
            arrival,
        })
    }
    
    /// Schedule the transmission of a bundle based on the contact data and available free intervals.
    ///
    /// This method shall be called after a dry run! Implementations might not ensure a clean behavior otherwise.
    /// The queue volume will be updated by this method: `true`.
    ///
    /// # Arguments
    ///
    /// * `contact_data` - Reference to the contact information.
    /// * `at_time` - The current time for scheduling purposes.
    /// * `bundle` - The bundle to be transmitted.
    ///
    /// # Returns
    ///
    /// Optionally returns `ContactManagerTxData` with transmission start and end times, or `None` if the bundle can't be transmitted.
    fn schedule_tx(
        &mut self,
        contact_data: &ContactInfo,
        at_time: Date,
        bundle: &Bundle,
    ) -> Option<ContactManagerTxData> {
        // First do a dry run to check if transmission is possible
        if let Some(data) = self.dry_run_tx(contact_data, at_time, bundle) {
            // Update MAV for the bundle's priority
            self.update_mav(bundle.size, bundle.priority);
            
            // Update queue size (auto_update is true)
            self.queue_size += bundle.size;
            
            return Some(data);
        }
        None
    }
    
    /// Initializes the EVL manager by setting the original volume based on contact duration, rate and priority.
    ///
    /// # Arguments
    ///
    /// * `contact_data` - Reference to the contact information.
    ///
    /// # Returns
    ///
    /// Returns `true` if initialization is successful.
    // fn try_init(&mut self, contact_data: &ContactInfo, bundle: &Bundle) -> bool {
    //     // Calculate maximum volume that can be transmitted during effective duration.
    //     let max_vol = (contact_data.end - contact_data.start) * self.rate;
        
    //     // Set the original_volume to contact Effective Volume Limit (C.EVL).
    //     self.original_volume = max_vol.min(self.get_mav(bundle.priority));      
    //     true
    // } // TODO: bundle input must exist?
    fn try_init(&mut self, contact_data: &ContactInfo, bundle: Option<&Bundle>) -> bool {
        // Calculate maximum volume that can be transmitted during effective duration.
        let max_vol = (contact_data.end - contact_data.start) * self.rate;
        
        // Set the original_volume to contact Effective Volume Limit (C.EVL).
        self.original_volume = match bundle {
            Some(b) => max_vol.min(self.get_mav(b.priority)),
            None => max_vol,
        };        
        true
    }

    
    /// Returns the original volume of the contact.
    ///
    /// # Returns
    ///
    /// A `Volume` representing the original volume.
    #[cfg(feature = "first_depleted")]
    fn get_original_volume(&self) -> Volume {
        self.original_volume
    }
}

/// Implements the DispatchParser to allow dynamic parsing. TODO： verify if needed.
impl crate::parsing::DispatchParser<EVLManager> for EVLManager {}

/// Implements the `Parser` trait for `EVLManager`, allowing the manager to be parsed from a lexer.
impl crate::parsing::Parser<EVLManager> for EVLManager {
    /// Parses an `EVLManager` from the lexer, extracting the rate and delay.
    ///
    /// # Arguments
    ///
    /// * `lexer` - The lexer used for parsing tokens.
    /// * `_sub` - An optional map for handling custom parsing logic (unused here).
    ///
    /// # Returns
    ///
    /// Returns a `ParsingState` indicating whether parsing was successful (`Finished`) or encountered an error (`Error`).
    fn parse(
        lexer: &mut dyn crate::parsing::Lexer,
    ) -> crate::parsing::ParsingState<Self> {
        let delay: Duration;
        let rate: DataRate;
        let original_mav: Vec<Volume>;

        let rate_state = <crate::types::DataRate as crate::types::Token<crate::types::DataRate>>::parse(lexer);
        match rate_state {
            crate::parsing::ParsingState::Finished(value) => rate = value,
            crate::parsing::ParsingState::Error(msg) => return crate::parsing::ParsingState::Error(msg),
            crate::parsing::ParsingState::EOF => {
                return crate::parsing::ParsingState::Error(format!(
                    "Parsing failed ({})",
                    lexer.get_current_position()
                ))
            }
        }

        let delay_state = <crate::types::Duration as crate::types::Token<crate::types::Duration>>::parse(lexer);
        match delay_state {
            crate::parsing::ParsingState::Finished(value) => delay = value,
            crate::parsing::ParsingState::Error(msg) => return crate::parsing::ParsingState::Error(msg),
            crate::parsing::ParsingState::EOF => {
                return crate::parsing::ParsingState::Error(format!(
                    "Parsing failed ({})",
                    lexer.get_current_position()
                ))
            }
        }

        // let mav_state = <Vec<Volume> as crate::types::Token<Vec<Volume>>>::parse(lexer);
        // TODO: modify all Vec<Volume> to VecWrapper<Volume> or do a conversion?
        let mav_state = <Vec<Volume> as crate::types::Token<crate::types::VecWrapper<Volume>>>::parse(lexer);
        match mav_state {
            crate::parsing::ParsingState::Finished(value) => original_mav = value,
            crate::parsing::ParsingState::Error(msg) => return crate::parsing::ParsingState::Error(msg),
            crate::parsing::ParsingState::EOF => {
                return crate::parsing::ParsingState::Error(format!(
                    "Parsing MAV failed ({})",
                    lexer.get_current_position()
                ))
            }
        }

        // Create and return the EVLManager
        crate::parsing::ParsingState::Finished(EVLManager::new(rate, delay, original_mav))
    }
}
