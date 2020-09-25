use infrared::sender::PulsetrainBuffer;
use infrared::protocols::nec::{NecCommand, NecStandard};
use infrared::BufferedReceiver;
use infrared::protocols::{Nec, Rc5, Rc6};
use infrared::Command;
use infrared::protocols::rc5::Rc5Command;
use infrared::protocols::rc6::Rc6Command;

#[test]
fn bufsend() {

    let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);

    //let ptb = PulsetrainBuffer::from(cmd);
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    ptb.load(&cmd);

    for ts in &ptb {
        println!("{:?}", ts);
    }

    let buf = ptb.into_iter().map(u32::from).collect::<Vec<_>>();

    let mut brecv: BufferedReceiver<Nec> = BufferedReceiver::new(&buf, 40_000);

    let cmd = brecv.next();
    println!("{:?}", cmd);
}

#[test]
fn test_bufsend_rc5() {
    let cmd: Rc5Command = Rc5Command::new(20, 10, false);

    //let ptb = PulsetrainBuffer::from(cmd);
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    ptb.load(&cmd);

    let buf = ptb.into_iter().map(u32::from).collect::<Vec<_>>();
    println!("{:?}", buf);

    let mut brecv: BufferedReceiver<Rc5> = BufferedReceiver::new(&buf, 40_000);

    let cmd = brecv.next();
    println!("{:?}", cmd);
}

#[test]
fn test_bufsend_rc6() {
    let cmd: Rc6Command = Rc6Command::new(70, 2);

    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    ptb.load(&cmd);

    let buf = ptb.into_iter().map(u32::from).collect::<Vec<_>>();
    println!("{:?}", buf);

    let mut brecv: BufferedReceiver<Rc6> = BufferedReceiver::new(&buf, 40_000);

    let cmdres = brecv.next().unwrap();
    assert_eq!(cmd.addr, cmdres.addr);
    assert_eq!(cmd.cmd, cmdres.cmd);
    println!("{:?}", cmd);
}

#[test]
fn test_samplerates() {

    let samplerates = [20_000, 40_000, 80_000];

    for samplerate in &samplerates {
        let mut ptb = PulsetrainBuffer::with_samplerate(*samplerate);

        let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);
        ptb.load(&cmd);

        let buf = ptb.into_iter().map(u32::from).collect::<Vec<_>>();
        let mut receiver: BufferedReceiver<Nec> = BufferedReceiver::new(&buf, *samplerate);

        if let Some(cmd) = receiver.next() {
            println!("{:?}", cmd);
            assert_eq!(cmd.address(), 20);
            assert_eq!(cmd.data(), 10);
        } else {
            panic!("Failed to parse command at samplerate: {}", samplerate)
        }
    }
}