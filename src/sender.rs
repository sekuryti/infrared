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

pub struct HalSender<PWMPIN, DUTY>
where
    PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pts: PulsetrainSender,
    pin: PWMPIN,
}

impl<'a, PWMPIN, DUTY> HalSender<PWMPIN, DUTY>
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{

    pub fn new(samplerate: u32, pin: PWMPIN) -> Self {
        Self {
            pts: PulsetrainSender::new(samplerate),
            pin
        }
    }

    pub fn tick(&mut self) {
        match self.pts.tick() {
            State::Idle => {
                self.pin.disable();
            }
            State::Transmit(enable) => {
                if enable {
                    self.pin.enable();
                } else {
                    self.pin.disable();
                }
            }
            State::Error => {
                self.pin.disable()
            }
        }
    }
}


pub struct PulsetrainSender {
    ptb: PulsetrainBuffer,
    index: usize,
    state: State,
    last: u32,
}

impl PulsetrainSender {

    pub fn new(samplerate: u32) -> Self {
        let ptb = PulsetrainBuffer::with_samplerate(samplerate);
        Self {
            ptb,
            index: 0,
            state: State::Idle,
            last: 0,
        }
    }

    /// Load command into internal buffer
    pub fn load(&mut self, c: impl Command) {
        self.ptb.load(c);
        self.state = State::Idle;
    }

    pub fn tick(&mut self, ts: u32) -> State {

        if let Some(dist) = self.ptb.get(self.index) {
            if ts.wrapping_sub(self.last) >= u32::from(dist) {

                let newstate = match self.state {
                    State::Idle | State::Transmit(false) => State::Transmit(true),
                    _ => State::Transmit(false),
                };

                self.index += 1;

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

    pub fn get(&self, index: usize) -> Option<u16> {
        self.buf.get(index).cloned()
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
        (self.index, Some(self.pulses.len()))
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
