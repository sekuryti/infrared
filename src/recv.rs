//! Receiver

use crate::{Command, protocols::ProtocolId};

/// Receiver state machine
pub trait Receiver {
    /// Protocol id
    const ID: ProtocolId;
    /// The resulting command type
    type Cmd: Command;

    /// Create a new Receiver of this type
    fn with_samplerate(samplerate: u32) -> Self;
    /// Add event to state machine
    fn event(&mut self, edge: bool, time: u32) -> State<Self::Cmd>;
    /// Reset receiver
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum State<CMD> {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done(CMD),
    /// Error while decoding
    Error(Error),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Receive error
pub enum Error {
    /// Error while decoding address
    Address(u32),
    /// Error decoding data bits
    Data(u32),
    /// Error receiver specific error
    Other(u32),
}

