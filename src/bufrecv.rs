use crate::{ReceiverSM, Command, recv::State};
use core::marker::PhantomData;

/// Receiver for decoding a captured pulse train
pub struct BufferReceiver<SM> {
    buf: [u16; 512],
    len: usize,
    scaler: u32,
    _sm: PhantomData<SM>,
}

pub struct BufferIterator<'a, SM> {
    buf: &'a [u16],
    pos: usize,
    scaler: u32,
    sm: SM,
}

impl<'a, SM: ReceiverSM> IntoIterator for &'a BufferReceiver<SM>  {
    type Item = SM::Cmd;
    type IntoIter = BufferIterator<'a, SM>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator {
            buf: &self.buf,
            scaler: self.scaler,
            pos: 0,
            sm: SM::create(),
        }
    }
}

impl<'a, SM: ReceiverSM> Iterator for BufferIterator<'a, SM> {
    type Item = SM::Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = u32::from(self.buf[self.pos]) * self.scaler;
            self.pos += 1;

            let state: State = self.sm.event(pos_edge, dt_us).into();

            match state {
                State::Idle | State::Receiving => {
                    continue;
                }
                State::Done => {
                    let cmd = self.sm.command();
                    self.sm.reset();
                    break cmd;
                }
                State::Error(_) => {
                    self.sm.reset();
                    break None;
                }
            }
        }
    }
}

impl<SM: ReceiverSM> BufferReceiver<SM> {
    pub fn new(pulses: &[u16], samplerate: u32) -> Self {
        let mut buf = [0; 512];
        buf[0..pulses.len()].copy_from_slice(pulses);
        let len = pulses.len();
        Self {
            buf,
            len,
            scaler: 1_000_000 / samplerate,
            _sm: PhantomData,
        }
    }

    pub fn add_cmd(&mut self, cmd: &impl Command) {
        let mut count = 0;
        cmd.to_pulsetrain(&mut self.buf[self.len..], &mut count);
        self.len += count;
    }

    pub fn add(&mut self, pulses: &[u16]) {
        self.buf[self.len..pulses.len()].copy_from_slice(pulses);
        self.len += pulses.len();
    }

    pub fn iter(&self) -> BufferIterator<'_, SM> {
        self.into_iter()
    }
}

