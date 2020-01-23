use crate::{
    Command,
    ProtocolId,
    recv::{Receiver, State},
};
use crate::protocols::utils::PulseWidthRange;

mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DenonState {
    Idle,
    Data(u8),
    Done,
}

#[derive(Debug)]
enum PulseWidth {
    SYNC,
    ZERO,
    ONE,
    FAIL,
}

impl Default for PulseWidth {
    fn default() -> Self {
        PulseWidth::FAIL
    }
}

impl From<usize> for PulseWidth {
    fn from(value: usize) -> Self {
        match value {
            0 => PulseWidth::SYNC,
            1 => PulseWidth::ZERO,
            2 => PulseWidth::ONE,
            _ => PulseWidth::FAIL,
        }
    }
}

pub struct Denon {
    state: DenonState,
    pub buf: u64,
    prev_rising: u32,
    prev_level: bool,
    ranges: PulseWidthRange<PulseWidth>,
}

pub struct Cmd {
    addr: u16,
    cmd: u8,
    pub raw: u64,
}

impl Command for Cmd {
    type Addr = u16;
    type Data = u8;

    fn construct(addr: u16, cmd: u8) -> Self {
        Self {
            addr, cmd, raw: 0,
        }
    }

    fn address(&self) -> u16 {
        self.addr
    }

    fn data(&self) -> Self::Data {
        self.cmd
    }
}

const fn nsamples(samplerate: u32) -> [(u32, u32); 4] {
    let period_time: u32 = 1000 / (samplerate / 1000);
    [
        // SYNC
        ((HEADER_HIGH + HEADER_LOW) / period_time, 5),
        // ZERO
        ((DATA_HIGH + ZERO_LOW) / period_time, 10),
        // ONE
        ((DATA_HIGH + ONE_LOW) / period_time, 10),
        // Not needed. Fix when const generics arrive
        (0xFFFFFFFF, 0),
    ]
}


impl Receiver for Denon {
    const ID: ProtocolId = ProtocolId::Denon;
    type Cmd = Cmd;

    fn with_samplerate(samplerate: u32) -> Self {

        let ranges = PulseWidthRange::new(&nsamples(samplerate));

        Denon {
            state: DenonState::Idle,
            buf: 0,
            prev_rising: 0,
            prev_level: false,
            ranges,
        }
    }

    fn event(&mut self, edge: bool, time: u32) -> State<Self::Cmd> {

        if edge {
            let samples = time.wrapping_sub(self.prev_rising);
            let pw = self.ranges.pulsewidth(samples);

            self.prev_rising = time;

            self.state = match (self.state, pw) {
                (DenonState::Idle, PulseWidth::SYNC) => DenonState::Data(0),
                (DenonState::Idle, _) => DenonState::Idle,
                (DenonState::Data(47), PulseWidth::ZERO) => DenonState::Done,
                (DenonState::Data(47), PulseWidth::ONE) => DenonState::Done,
                (DenonState::Data(idx), PulseWidth::ZERO) => DenonState::Data(idx + 1),
                (DenonState::Data(idx), PulseWidth::ONE) => {
                    self.buf |= 1 << idx;
                    DenonState::Data(idx + 1)
                },
                (DenonState::Data(idx), _) => DenonState::Idle,
                (DenonState::Done, _) => DenonState::Done,
            }
        }

        if self.state == DenonState::Done {
            return State::Done(
                Cmd {
                    addr: 0,
                    cmd: 0,
                    raw: self.buf,
                }
            );
        }

        State::Idle
    }

    fn reset(&mut self) {
        self.state = DenonState::Idle;
        self.buf = 0;
        self.prev_rising = 0;
        self.prev_level = false;
    }
}