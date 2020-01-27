//! Transmitter state machine
//!

#[derive(Debug)]
/// Transmitter state
pub enum State {
    /// Transmitter is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error state
    Error,
}

/// Transmitter
pub trait Statemachine<CMD> {
    /// Load command into transmitter
    fn load(&mut self, cmd: CMD);
    /// Step the transfer loop
    fn step(&mut self, ts: u32) -> State;
    /// Reset the transmitter
    fn reset(&mut self);
}

#[cfg(feature = "embedded-hal")]
/// Embedded hal pwm transmitter
pub trait Transmitter<CMD>: Statemachine<CMD> {
    /// Step the transmit loop and output on `pwm`
    fn pwmstep<PWMPIN, DUTY>(&mut self, ts: u32, pwm: &mut PWMPIN) -> State
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
