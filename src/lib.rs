//! Infrared

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod recv;
pub mod send;
pub mod protocols;
pub use protocols::ProtocolId;

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "embedded-hal")]
mod hal;

#[cfg(feature = "embedded-hal")]
pub use hal::{
    IrReceiver,
    IrReceiver2,
    IrReceiver3,
    IrReceiver4,
    IrReceiver5,
};

/// Remote control command trait
pub trait Command {
    type Addr;
    type Data;

    /// Constuct a command
    fn construct(addr: Self::Addr, data: Self::Data) -> Self;

    /// Command address
    fn address(&self) -> Self::Addr;

    /// Get the data associated with the command
    fn data(&self) -> Self::Data;
}

