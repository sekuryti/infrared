use core::ops::Range;

use crate::{
    ProtocolId,
    recv::{
        Receiver,
        State,
        Error,
    },
    protocols::rc5::Rc5Command,
};

pub struct Rc5 {
    samplerate: u32,
    state: Rc5State,
    pinval: bool,
    bitbuf: u16,
    last: u32,
    pub rc5cntr: u32,
}

impl Rc5 {
    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            state: Rc5State::Idle,
            pinval: false,
            bitbuf: 0,
            rc5cntr: 0,
        }
    }

    pub fn interval_to_units(&self, interval: u16) -> Option<u32> {
        for i in 1..=2 {
            if rc5_multiplier(self.samplerate, i).contains(&(u32::from(interval))) {
                return Some(i);
            }
        }
        None
    }

    // Time since last edge
    fn delta(&self, sampletime: u32) -> u16 {
        let delta = sampletime.wrapping_sub(self.last);
        if delta >= core::u16::MAX.into() {
            return 0;
        }
        delta as u16
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5State {
    Idle,
    Data(u8),
    Done,
    Err(Error),
}

const RISING: bool = true;
const FALLING: bool = false;

type Rc5Res = State<Rc5Command>;

impl Receiver for Rc5 {
    const ID: ProtocolId = ProtocolId::Rc5;
    type Cmd = Rc5Command;

    fn with_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate)
    }

    fn event(&mut self, rising: bool, sampletime: u32) -> Rc5Res {
        use Rc5State::*;

        let delta = self.delta(sampletime);
        self.last = sampletime;
        self.pinval = rising;

        // Number of rc5 units since last pin edge
        let rc5units = self.interval_to_units(delta);

        if let Some(units) = rc5units {
            self.rc5cntr += units;
        }

        let odd = self.rc5cntr & 1 == 0;

        let newstate = match (self.state, rising, rc5units) {
            (Idle, FALLING, _) => Idle,
            (Idle, RISING, _) => {
                self.bitbuf |= 1 << 13;
                Data(12)
            }

            (Data(0), RISING, Some(_)) if odd => {
                self.bitbuf |= 1;
                Done
            }
            (Data(0), FALLING, Some(_)) if odd => Done,

            (Data(bit), RISING, Some(_)) if odd => {
                self.bitbuf |= 1 << bit;
                Data(bit - 1)
            }
            (Data(bit), FALLING, Some(_)) if odd => Data(bit - 1),

            (Data(bit), _, Some(_)) => Data(bit),
            (Data(_), _, None) => Err(Error::Data(delta as u32)),
            (Done, _, _) => Done,
            (Err(err), _, _) => Err(err),
        };

        self.state = newstate;

        match self.state {
            Idle => State::Idle,
            Done => State::Done(Rc5Command::from_bits(self.bitbuf)),
            Err(err) => State::Error(err),
            _ => State::Receiving,
        }
    }

    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.pinval = false;
        self.bitbuf = 0;
        self.rc5cntr = 0;
    }
}

const fn rc5_multiplier(samplerate: u32, multiplier: u32) -> Range<u32> {
    let base = (samplerate * 889 * multiplier) / 1_000_000;
    range(base, 10)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

    Range {
        start: len - tol - 2,
        end: len + tol + 2,
    }
}
