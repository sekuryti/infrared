use core::marker::PhantomData;
use crate::{
    transmitter::{Statemachine, State},
    protocols::nec::{NecCommand, NecTiming, NecVariant}
};

enum TransmitStateInternal {
    Idle,
    Start,
    HeaderHigh,
    HeaderLow,
    DataLow(u32),
    DataHigh(u32),
    Done,
}

pub struct NecTypeTransmitter<N> {
    state: TransmitStateInternal,
    samples: NSamples,
    last_ts: u32,
    cmd: u32,
    nectype: PhantomData<N>,
}

struct NSamples {
    hh: u32,
    hl: u32,
    data: u32,
    zero: u32,
    one: u32,
}

impl<N: NecVariant> NecTypeTransmitter<N> {
    pub fn new(samplerate: u32) -> Self {
        let period: u32 = (1 * 1000) / (samplerate / 1000);

        let samples = NSamples::new(period, &N::TIMING);
        Self {
            state: TransmitStateInternal::Idle,
            samples,
            last_ts: 0,
            cmd: 0,
            nectype: PhantomData,
        }
    }
}

impl<N: NecVariant> Statemachine<NecCommand> for NecTypeTransmitter<N> {
    fn load(&mut self, cmd: NecCommand) {
        self.cmd = N::encode_command(cmd);
        self.state = TransmitStateInternal::Start;
    }

    fn step(&mut self, ts: u32) -> State {
        use TransmitStateInternal::*;

        let interval = ts.wrapping_sub(self.last_ts);

        self.state = match self.state {
            Start => {
                self.last_ts = ts;
                HeaderHigh
            }
            HeaderHigh => {
                if interval >= self.samples.hh {
                    self.last_ts = ts;
                    HeaderLow
                } else {
                    HeaderHigh
                }
            }
            HeaderLow => {
                if interval >= self.samples.hl {
                    self.last_ts = ts;
                    DataHigh(0)
                } else {
                    HeaderLow
                }
            }

            DataHigh(bidx) => {
                if interval >= self.samples.data {
                    self.last_ts = ts;
                    DataLow(bidx)
                } else {
                    DataHigh(bidx)
                }
            }
            DataLow(32) => Done,
            DataLow(bidx) => {
                let samples = if (self.cmd & (1 << bidx)) != 0 {
                    self.samples.one
                } else {
                    self.samples.zero
                };

                if interval >= samples {
                    self.last_ts = ts;
                    DataHigh(bidx + 1)
                } else {
                    DataLow(bidx)
                }
            }
            Done => Done,
            Idle => Idle,
        };

        match self.state {
            HeaderHigh | DataHigh(_)    => State::Transmit(true),
            HeaderLow | DataLow(_)      => State::Transmit(false),
            Done | Idle | Start         => State::Idle,
        }
    }

    fn reset(&mut self) {
        self.cmd = 0;
        self.state = TransmitStateInternal::Idle;
        self.last_ts = 0;
    }
}

#[cfg(feature = "embedded-hal")]
impl<N: NecVariant> crate::transmitter::Transmitter<NecCommand> for NecTypeTransmitter<N> {}

impl NSamples {
    pub const fn new(period: u32, pulsedistance: &NecTiming) -> Self {
        Self {
            hh: pulsedistance.hh / period,
            hl: pulsedistance.hl / period,
            zero: pulsedistance.zl / period,
            data: pulsedistance.dh / period,
            one: pulsedistance.ol / period,
        }
    }
}
