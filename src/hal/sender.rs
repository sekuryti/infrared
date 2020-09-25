use crate::sender::{PulsetrainSender, State};
use crate::Command;

pub struct HalSender<PWMPIN, DUTY>
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pts: PulsetrainSender,
    pin: PWMPIN,
    pub counter: u32,
}

impl<'a, PWMPIN, DUTY> HalSender<PWMPIN, DUTY>
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pub fn new(samplerate: u32, pin: PWMPIN) -> Self {
        Self {
            pts: PulsetrainSender::new(samplerate),
            pin,
            counter: 0,
        }
    }

    pub fn load<C: Command>(&mut self, cmd: &C) {
        if self.pts.state == State::Idle {
            self.pts.load(cmd);
        }
    }

    pub fn tick(&mut self) {
        self.counter += 1;
        match self.pts.tick(self.counter) {
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

