//! Transmitter state machine
//!

use crate::Command;
use core::marker::PhantomData;
use core::convert::TryFrom;

#[derive(Debug, PartialEq, Copy, Clone)]
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

pub struct HalSender<'a, PWMPIN, DUTY>
where
    PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pts: PulsetrainSender<'a>,
    pin: PWMPIN,
}

impl<'a, PWMPIN, DUTY> HalSender<'a, PWMPIN, DUTY>
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{

    pub fn new(samplerate: u32, pin: PWMPIN) -> Self {
        Self {
            pts: PulsetrainSender::new(samplerate),
            pin
        }
    }
}


pub struct PulsetrainSender<'a> {
    ptb: PulsetrainBuffer,
    iter: PulsetrainIterator<'a>,
    /// Last state change
    last: u32,
    /// Delta samples to next state change
    dist: Option<u16>,
    state: State,
}

impl<'a> PulsetrainSender<'a> {

    pub fn new(samplerate: u32) -> Self {
        let ptb = PulsetrainBuffer::with_samplerate(samplerate);
        Self {
            ptb,
            iter: PulsetrainIterator {
                pulses: &[],
                index: 0
            },
            last: 0,
            dist: None,
            state: State::Idle
        }
    }

    pub fn load(&'a mut self, c: impl Command) {
        self.ptb.load(c);
        self.iter = self.ptb.into_iter();
        self.dist = self.iter.next();
        self.state = State::Idle;
    }

    pub fn poll(&mut self, ts: u32) -> State {

        if let Some(dt) = self.dist {
            if ts - self.last >= dt.into() {
                // State change
                let newstate = match self.state {
                    State::Idle | State::Transmit(false) => State::Transmit(true),
                    _ => State::Transmit(false),
                };

                self.dist = self.iter.next();

                if self.state != newstate {
                    self.last = ts;
                }
            }
        } else {
            self.state = State::Idle;
        }

        self.state
    }
}

pub struct PulsetrainBuffer {
    pub buf: [u16; 96],
    pub len: usize,
    pub scaler: u16,
}

impl PulsetrainBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; 96],
            len: 0,
            scaler: 1,
        }
    }

    pub fn with_samplerate(samplerate: u32) -> Self {
        Self {
            buf: [0; 96],
            len: 0,
            scaler: u16::try_from(1000 / (samplerate / 1000)).unwrap(),
        }
    }

    pub fn load(&mut self, c: impl Command) {
        c.pulsetrain(&mut self.buf, &mut self.len);

        // Apply the scaling on the buf
        for i in 0 .. self.len {
            self.buf[i] = self.buf[i] / self.scaler;
        }
    }
}

impl<C: Command> From<C> for PulsetrainBuffer {
    fn from(c: C) -> Self {
        let mut ptb = PulsetrainBuffer::new();
        c.pulsetrain(&mut ptb.buf, &mut ptb.len);
        ptb
    }
}

impl<'a> IntoIterator for &'a PulsetrainBuffer {
    type Item = u16;
    type IntoIter = PulsetrainIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PulsetrainIterator {
            pulses: &self.buf[0..self.len],
            index: 0
        }
    }
}

pub struct PulsetrainIterator<'a> {
    pulses: &'a [u16],
    index: usize,
}

impl<'a> Iterator for PulsetrainIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.pulses.len() {
            None
        } else {
            let r = self.pulses[self.index];
            self.index += 1;
            Some(r)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.pulses.len()))
    }
}

pub struct PwmSender<C: Command> {
    cmd: Option<C>,
}

impl<C: Command> PwmSender<C> {

    pub fn load(&mut self, c: C) {
        self.cmd = Some(c);
    }
}
