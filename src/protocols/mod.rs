//! Infrared protocols

#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "sbp")]
pub mod sbp;

/// Capture
pub mod capture;

pub(crate) mod utils;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Remote Control Protocol Id
pub enum ProtocolId {
    /// Nec
    Nec = 1,
    /// Nec with 16 bit address
    Nec16 = 2,
    /// Nec - Samsung variant
    NecSamsung = 3,
    /// Philips Rc5
    Rc5 = 4,
    /// Philips Rc6
    Rc6 = 5,
    /// Samsung 36 bit protocol
    Sbp = 6,

    /// Logging
    Logging = 31,
}



