use infrared::sender::PulsetrainBuffer;
use infrared::protocols::nec::{NecCommand, NecStandard};
use infrared::BufferedReceiver;
use infrared::protocols::Nec;
use infrared::Command;

#[test]
fn bufsend() {

    let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);

    //let ptb = PulsetrainBuffer::from(cmd);
    let mut ptb = PulsetrainBuffer::with_samplerate(40_000);

    ptb.load(cmd);

    for ts in &ptb {
        println!("{:?}", ts);
    }

    let buf = ptb.into_iter().map(u32::from).collect::<Vec<_>>();

    let mut brecv: BufferedReceiver<Nec> = BufferedReceiver::new(&buf, 40_000);

    let cmd = brecv.next();
    println!("{:?}", cmd);
}

#[test]
fn test_samplerates() {

    let samplerates = [20_000, 40_000, 80_000];

    for samplerate in &samplerates {
        let mut ptb = PulsetrainBuffer::with_samplerate(*samplerate);

        let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);
        ptb.load(cmd);

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