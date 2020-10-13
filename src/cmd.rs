/// Remote control command trait
pub trait Command {
    /// Constuct a command
    fn construct(addr: u32, data: u32) -> Option<Self>
    where
        Self: core::marker::Sized;

    /// Command address
    fn address(&self) -> u32;

    /// Get the data associated with the command
    fn data(&self) -> u32;

    /// Protocol
    fn protocol(&self) -> Protocol {
        Protocol::Unknown
    }

    fn to_pulsetrain(&self, _buf: &mut [u16], _len: &mut usize) {}
}

#[derive(Debug, Copy, Clone)]
/// Protocol
pub enum Protocol {
    Nec,
    Nec16,
    NecSamsung,
    Rc5,
    Rc6,
    Sbp,
    Unknown,
}
