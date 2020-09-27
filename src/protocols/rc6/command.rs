use crate::{Command, Protocol};
use core::convert::TryInto;

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
