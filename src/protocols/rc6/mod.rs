//! Rc6

use core::convert::TryInto;
use core::ops::Range;

use crate::{
    cmd::Protocol,
    recv::{Error, ReceiverSM, State},
    Command,
};

mod tests;

#[derive(Debug)]
pub struct Rc6Command {
    pub mode: u8,
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Command {
    pub fn new(addr: u8, cmd: u8) -> Self {
        Self {
            mode: 0,
            addr,
            cmd,
            toggle: false,
        }
    }

    pub fn from_bits(bits: u32, toggle: bool) -> Self {
        let addr = (bits >> 8) as u8;
        let cmd = (bits & 0xFF) as u8;
        Self { mode: 0, addr, cmd, toggle }
    }

    pub fn to_bits(&self) -> u32 {
        // MMMT_AAAAAAAA_CCCCCCCC
        u32::from(self.addr) << 8 | u32::from(self.cmd)
    }
}

impl Command for Rc6Command {
    fn construct(addr: u32, cmd: u32) -> Option<Self> {
        Some(Rc6Command::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn data(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Rc6
    }

    fn pulsetrain(&self, buf: &mut [u16], len: &mut usize) {

        // Leader
        buf[0] = 0;
        buf[1] = 6 * 444;
        // Start bit after leading pause
        buf[2] = 2 * 444;
        buf[3] = 444;

        // Mode 000
        buf[4] = 2 * 444;
        buf[5] = 444;
        buf[6] = 444;
        buf[7] = 444;
        buf[8] = 444;
        buf[9] = 444;
        // Toggle
        //  TODO: Add toggle to command and make this configurable
        buf[10] = 2 * 444;

        let bits = self.to_bits();
        let mut prev;
        let mut index;

        let first_bit_set = bits & (1 << 15) != 0;

        if first_bit_set {
            buf[11] = 2 * 444 + 444;
            index = 12;
            prev = true;
        } else {
            buf[11] = 2 * 444;
            buf[12] = 444;
            index = 13;
            prev = false;
        }

        for b in 1..16 {
            let cur = bits & (1 << (15 - b)) != 0;

            if prev == cur {
                buf[index] = 444;
                buf[index+1] = 444;
                index += 2;
            } else {
                buf[index] = 444 * 2;
                index += 1;
            }

            prev = cur;
        }

        // Terminate
        if prev == false {
            buf[index] = 2 * 444;
            index += 1;
        }

        *len = index;
    }
}

#[derive(Default)]
pub struct Rc6 {
    state: Rc6State,
    data: u32,
    headerdata: u32,
    toggle: bool,
    rc6_clock: u32,
}

impl Rc6 {
    pub fn interval_to_units(interval: u16) -> Option<u32> {
        let interval = u32::from(interval);

        for i in 1..=6 {
            if rc6_multiplier(i).contains(&interval) {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Rc6Err(Error),
}

impl Default for Rc6State {
    fn default() -> Self {
        Rc6State::Idle
    }
}

impl From<Rc6State> for State {
    fn from(state: Rc6State) -> Self {
        use Rc6State::*;
        match state {
            Idle => State::Idle,
            Done => State::Done,
            Rc6Err(err) => State::Error(err),
            Leading | LeadingPaus | HeaderData(_) | Trailing | Data(_) => State::Receiving,
        }
    }
}

const RISING: bool = true;
const FALLING: bool = false;

impl ReceiverSM for Rc6 {
    type Cmd = Rc6Command;
    type InternalState = Rc6State;

    fn create() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Rc6State {
        use Rc6State::*;

        // Number of rc6 units since last pin edge
        let n_units = Rc6::interval_to_units(dt as u16);

        if let Some(units) = n_units {
            self.rc6_clock += units;
        } else {
            self.reset();
        }

        let odd = self.rc6_clock & 1 == 1;

        self.state = match (self.state, rising, n_units) {
            (Idle,          FALLING,    _)          => Idle,
            (Idle,          RISING,     _)          => { self.rc6_clock = 0; Leading },
            (Leading,       FALLING,    Some(6))    => LeadingPaus,
            (Leading,       _,          _)          => Idle,
            (LeadingPaus,   RISING,     Some(2))    => HeaderData(3),
            (LeadingPaus,   _,          _)          => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                self.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      FALLING,    Some(3))    => { self.toggle = false; Data(15) }
            (Trailing,      RISING,     Some(2))    => { self.toggle = true; Data(15) }
            (Trailing,      FALLING,    Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       RISING,     Some(_)) if odd => Done,
            (Data(0),       FALLING,    Some(_)) if odd => { self.data |= 1; Done }
            (Data(0),       _,          Some(_))    => Data(0),
            (Data(n),       RISING,     Some(_)) if odd => Data(n - 1),
            (Data(n),       FALLING,    Some(_)) if odd => { self.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))    => Data(n),
            (Data(_),       _,          None)       => Rc6Err(Error::Data),

            (Done,          _,          _)          => Done,
            (Rc6Err(err),    _,          _)         => Rc6Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(self.data, self.toggle))
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.rc6_clock = 0;
    }
}

const fn rc6_multiplier(multiplier: u32) -> Range<u32> {
    let base = 444 * multiplier;
    range(base, 12)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

    Range {
        start: len - tol - 2,
        end: len + tol + 4,
    }
}

