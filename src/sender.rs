//! Transmitter state machine
//!

use crate::Command;
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

pub struct PulsetrainSender {
    ptb: PulsetrainBuffer,
    index: usize,
    pub(crate) state: State,
    ts_lastedge: u32,
}

impl PulsetrainSender {

    pub fn new(samplerate: u32) -> Self {
        let ptb = PulsetrainBuffer::with_samplerate(samplerate);
        Self {
            ptb,
            index: 0,
            state: State::Idle,
            ts_lastedge: 0,
        }
    }

    /// Load command into internal buffer
    pub fn load(&mut self, c: &impl Command) {
        self.ptb.load(c);
        self.state = State::Idle;
    }

    pub fn tick(&mut self, ts: u32) -> State {

        if let Some(dist) = self.ptb.get(self.index) {
            if ts.wrapping_sub(self.ts_lastedge) >= u32::from(dist) {

                let newstate = match self.state {
                    State::Idle | State::Transmit(false) => State::Transmit(true),
                    _ => State::Transmit(false),
                };

                self.index += 1;

                if self.state != newstate {
                    self.ts_lastedge = ts;
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

    pub fn load(&mut self, c: &impl Command) {
        c.to_pulsetrain(&mut self.buf, &mut self.len);

        // Apply the scaling on the buf
        for elem in &mut self.buf[..self.len] {
            *elem /= self.scaler;
        }
    }

    pub fn get(&self, index: usize) -> Option<u16> {
        self.buf.get(index).cloned()
    }
}

impl<C: Command> From<C> for PulsetrainBuffer {
    fn from(c: C) -> Self {
        let mut ptb = PulsetrainBuffer::new();
        c.to_pulsetrain(&mut ptb.buf, &mut ptb.len);
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

