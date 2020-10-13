use infrared::sender::PulsetrainBuffer;
use infrared::protocols::nec::{NecCommand, NecStandard};
use infrared::BufferReceiver;
use infrared::protocols::{Nec, Rc5, Rc6};
use infrared::Command;
use infrared::protocols::rc5::Rc5Command;
use infrared::protocols::rc6::Rc6Command;

#[test]
fn nec() {
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    for address in 0..255 {
        for cmdnum in 0..255 {
            let cmd: NecCommand<NecStandard> = NecCommand::new(address, cmdnum);
            ptb.load(&cmd);
            let brecv: BufferReceiver<Nec> = BufferReceiver::new(&ptb.buf, 40_000);

            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

#[test]
fn rc5() {
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    for address in 0..32 {
        for cmdnum in 0..64 {
            let cmd: Rc5Command = Rc5Command::new(address, cmdnum, false);
            ptb.load(&cmd);
            let brecv: BufferReceiver<Rc5> = BufferReceiver::new(&ptb.buf, 40_000);
            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

#[test]
fn rc6() {
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    for address in 0..255 {
        for cmdnum in 0..255 {
            let cmd: Rc6Command = Rc6Command::new(address, cmdnum);
            ptb.load(&cmd);
            let brecv: BufferReceiver<Rc6> = BufferReceiver::new(&ptb.buf, 40_000);
            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

#[test]
fn test_samplerates() {

    let samplerates = [20_000, 40_000, 80_000];

    for samplerate in &samplerates {
        let mut ptb = PulsetrainBuffer::with_samplerate(*samplerate);

        let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);
        ptb.load(&cmd);

        let receiver: BufferReceiver<Nec> = BufferReceiver::new(&ptb.buf, *samplerate);

        if let Some(cmd) = receiver.iter().next() {
            println!("{:?}", cmd);
            assert_eq!(cmd.address(), 20);
            assert_eq!(cmd.data(), 10);
        } else {
            panic!("Failed to parse command at samplerate: {}", samplerate)
        }
    }
}