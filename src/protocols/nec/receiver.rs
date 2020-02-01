use crate::{
    ProtocolId,
    protocols::nec::{NecCommand, NecVariant, NecTiming},
    protocols::utils::PulseWidthRange,
    recv::{Error, State, Receiver},
};

/// Generic type for Nec Receiver
pub struct NecType<N> {
    // State
    state: NecState,
    // Time of last event
    last_event: u32,
    // Data buffer
    pub bitbuf: u32,
    // Timing and tolerances
    ranges: PulseWidthRange<PulseWidth>,
    // Last command (used by repeat)
    lastcommand: u32,
    // The type of Nec
    nectype: core::marker::PhantomData<N>,
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum NecState {
    // Waiting for first pulse
    Init,
    // Receiving data
    Receiving(u32),
    // Command received
    Done,
    // Repeat command received
    RepeatDone,
    // In error state
    Err(Error),
}

impl<N: NecVariant> NecType<N> {
    pub fn new(samplerate: u32) -> Self {
        let timing = N::TIMING;
        Self::with_timing(samplerate, timing)
    }

    fn with_timing(samplerate: u32, timing: &NecTiming) -> Self {
        let nsamples = nsamples_from_timing(timing, samplerate);
        let ranges = PulseWidthRange::new(&nsamples);

        Self {
            state: NecState::Init,
            last_event: 0,
            bitbuf: 0,
            lastcommand: 0,
            nectype: core::marker::PhantomData,
            ranges,
        }
    }
}

impl<N: NecVariant> Receiver for NecType<N> {
    const ID: ProtocolId = N::PROTOCOL;
    type Cmd = NecCommand;

    fn with_samplerate(samplerate: u32) -> Self {
        let timing = N::TIMING;
        Self::with_timing(samplerate, timing)
    }

    fn event(&mut self, rising: bool, time: u32) -> State<Self::Cmd> {
        use NecState::*;
        use PulseWidth::*;

        if rising {
            // Calculate the nbr of samples since last rising edge
            let nsamples = time.wrapping_sub(self.last_event);

            // Map the nbr of samples to a pulsewidth
            let pulsewidth = self.ranges.pulsewidth(nsamples);

            let newstate = match (self.state, pulsewidth) {
                (Init, Sync) => Receiving(0),
                (Init, Repeat) => RepeatDone,
                (Init, _) => Init,

                (Receiving(31), One) => { self.bitbuf |= 1 << 31; Done }
                (Receiving(31), Zero) => Done,

                (Receiving(bit), One) => { self.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit), Zero) => Receiving(bit + 1),

                (Receiving(_), _) => Err(Error::Data(0)),

                (Done, _) => Done,
                (RepeatDone, _) => RepeatDone,
                (Err(err), _) => Err(err),
            };

            self.last_event = time;
            self.state = newstate;
        }

        match self.state {
            Init => State::Idle,
            Done => State::Done(N::decode_command(self.bitbuf)),
            RepeatDone => State::Done(N::decode_command(self.lastcommand)),
            Err(e) => State::Error(e),
            _ => State::Receiving,
        }
    }

    fn reset(&mut self) {
        self.state = NecState::Init;
        self.last_event = 0;
        self.lastcommand = if self.bitbuf == 0 {
            self.lastcommand
        } else {
            self.bitbuf
        };
        self.bitbuf = 0;
    }
}

#[derive(Debug, Clone)]
pub enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    NotAPulseWidth = 4,
}

impl Default for PulseWidth {
    fn default() -> Self {
        PulseWidth::NotAPulseWidth
    }
}

impl From<usize> for PulseWidth {
    fn from(v: usize) -> Self {
        match v {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Repeat,
            2 => PulseWidth::Zero,
            3 => PulseWidth::One,
            _ => PulseWidth::NotAPulseWidth,
        }
    }
}

const fn nsamples_from_timing(t: &NecTiming, samplerate: u32) -> [(u32, u32); 4] {
    let per: u32 = 1000 / (samplerate / 1000);
    [
        ((t.hh + t.hl) / per, 5),
        ((t.hh + t.rl) / per, 5),
        ((t.dh + t.zl) / per, 10),
        ((t.dh + t.ol) / per, 10),
    ]
}
