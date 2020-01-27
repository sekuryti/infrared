//! Receiver state machine
//!

use crate::{Command, protocols::ProtocolId};

/// Receiver state machine
pub trait Statemachine {
    /// Protocol id
    const ID: ProtocolId;
    /// The resulting command type
    type Cmd: Command;

    /// New
    fn with_samplerate(samplerate: u32) -> Self;
    /// Add event to state machine
    fn event(&mut self, edge: bool, time: u32) -> State<Self::Cmd>;
    /// Reset receiver
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum State<CMD> {
    Idle,
    Receiving,
    Done(CMD),
    Error(Error),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Receiver error
pub enum Error {
    Address(u32),
    Data(u32),
    Other(u32),
}

