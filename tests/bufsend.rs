use infrared::sender::BufSender;
use infrared::protocols::nec::{NecCommand, NecStandard};
use infrared::BufferedReceiver;
use infrared::protocols::Nec;

#[test]
fn bufsend() {

    let mut bs = BufSender::new();

    let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);

    bs.to_pulsetrain(cmd);

    for ts in bs.buf.iter() {
        println!("{:?}", ts);
    }

    let buf = bs.buf.iter().map(|v| u32::from(*v)).collect::<Vec<_>>();

    let mut brecv: BufferedReceiver<Nec> = BufferedReceiver::new(&buf, 1_000_000);

    let cmd = brecv.next();
    println!("{:?}", cmd);
}