//! Transmitter state machine
//!

use crate::Command;
use core::marker::PhantomData;

#[derive(Debug)]
/// Sender state
pub enum State {
    /// Sender is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error
    Error,
}

/// Sender
pub trait Sender<CMD> {
    /// Load command into the sender
    fn load(&mut self, cmd: CMD);
    /// Step the transfer loop
    fn step(&mut self, ts: u32) -> State;
    /// Reset the transmitter
    fn reset(&mut self);
}

#[cfg(feature = "embedded-hal")]
/// Embedded hal IR Sender
pub trait PwmPinSender<CMD>: Sender<CMD> {
    /// Step the transmit loop and output on `pwm`
    fn step_pwm<PWMPIN, DUTY>(&mut self, ts: u32, pwm: &mut PWMPIN) -> State
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
    {
        let state = self.step(ts);
        match state {
            State::Transmit(true) => pwm.enable(),
            _ => pwm.disable(),
        }
        state
    }
}

pub struct BufSender {
    pub buf: [u16; 96],
    pub len: usize,
}

impl BufSender {

    pub fn new() -> Self {
        Self {
            buf: [0; 96],
            len: usize,
        }
    }

    pub fn to_pulsetrain<C: Command>(&mut self, c: C) {
        c.to_pulsetrain(&mut self.buf);
    }
}

pub struct PwmSender<C: Command> {
    cmd: Option<C>,
}

impl<C: Command> PwmSender<C> {

    pub fn load(&mut self, c: CMD) {
        self.cmd = Some(c);
    }
}
