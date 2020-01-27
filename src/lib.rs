//! Infrared

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

/// Remote control command trait
pub trait Command {
    fn construct(addr: u32, cmd: u32) -> Self;
    /// Command address
    fn address(&self) -> u32;
    /// Get the data associated with the command
    fn data(&self) -> u32;
}

#[cfg(feature = "embedded-hal")]
mod hal;

pub mod receiver;
pub mod transmitter;
pub mod protocols;
pub use protocols::ProtocolId;

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "embedded-hal")]
pub use hal::{
    InfraredReceiver,
    InfraredReceiver2,
    InfraredReceiver3,
    InfraredReceiver4,
    InfraredReceiver5,
};

