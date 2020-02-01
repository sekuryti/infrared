use embedded_hal::digital::v2::InputPin;

use crate::{
    recv::{State, Receiver},
};


/// Embedded hal Receiever
pub struct IrReceiver<RECV, PIN> {
    /// The receiver state machine
    recv: RECV,
    /// The pin used
    pin: PIN,
    pinval: bool,
}

impl<CMD, PIN, PINERR, RECV> IrReceiver<RECV, PIN>
where
    CMD: crate::Command,
    RECV: Receiver<Cmd = CMD>,
    PIN: InputPin<Error = PINERR>,
{
    pub fn with_receiver(recv: RECV, pin: PIN) -> Self {
        Self {
            recv,
            pin,
            pinval: false,
        }
    }

    pub fn new(pin: PIN, samplerate: u32) -> Self {
        Self {
            recv: RECV::with_samplerate(samplerate),
            pin,
            pinval: false,
        }
    }

    pub fn destroy(self) -> PIN {
        self.pin
    }

    pub fn sample(&mut self, sample: u32) -> Result<Option<CMD>, PINERR> {
        let pinval = self.pin.is_low()?;

        if self.pinval != pinval {
            let r = self.recv.event(pinval, sample);

            if let State::Done(cmd) = r {
                self.recv.reset();
                return Ok(Some(cmd));
            }

            if let State::Error(_err) = r {
                self.recv.reset();
            }

            self.pinval = pinval;
        }

        Ok(None)
    }

    #[cfg(feature = "remotes")]
    pub fn sample_as_button<RC>(&mut self, sampletime: u32) -> Result<Option<RC::Button>, PINERR>
    where
        RC: crate::remotes::RemoteControl<Command = CMD>,
    {
        self.sample(sampletime)
            .map(|opt| opt.and_then(RC::decode))
    }
}

macro_rules! create_receiver {
($name:ident, [ $( ($N:ident, $P:ident, $C:ident) ),* ]) =>
{
    /// HAL receiver
    pub struct $name<$( $P ),* , PIN> {
        pin: PIN,
        pinval: bool,
        $( $N : $P ),*
    }

    impl<PIN, PINERR, $( $P, $C ),* > $name <$( $P ),* , PIN>
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: Receiver<Cmd = $C>),*,
        $( $C: crate::Command ),*,
    {
        pub fn with_receivers($( $N : $P ),* , pin: PIN) -> Self {
            Self {
                pin,
                pinval: false,
                $( $N ),*,
            }
        }

        pub fn new(pin: PIN, samplerate: u32) -> Self {
            Self {
                pin,
                pinval: false,
                $( $N: $P::with_samplerate(samplerate)),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn step(&mut self, ts: u32) -> Result<( $( Option<$C>),*), PINERR> {
            let pinval = self.pin.is_low()?;

            if self.pinval != pinval {
                self.pinval = pinval;

                Ok(($(
                    match self.$N.event(pinval, ts) {
                        State::Done(cmd) => {
                            self.$N.reset();
                            Some(cmd)
                        },
                        State::Error(_) => {
                            self.$N.reset();
                            None
                        }
                        _ => None,
                    }
                    ),* ))
                } else {
                    Ok(Default::default())
                }
            }
        }

    };
}

create_receiver!(IrReceiver2, [(recv1, RECV1, CMD1), (recv2, RECV2, CMD2)]);

create_receiver!(
    IrReceiver3,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3)
    ]
);

create_receiver!(
    IrReceiver4,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4)
    ]
);
create_receiver!(
    IrReceiver5,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4),
        (recv5, RECV5, CMD5)
    ]
);
