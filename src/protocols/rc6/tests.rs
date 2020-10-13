use crate::protocols::rc6::{Rc6, Rc6Command};
use crate::recv::*;
use crate::{Command, BufferReceiver};

#[test]
fn pulsetrain_on_chain() {

    let mut recv: BufferReceiver<Rc6> = BufferReceiver::new(&[], 1_000_000);
    let cmds = [
        Rc6Command::new(72, 17),
        Rc6Command::new(0, 21),
        Rc6Command::new(1, 33)
    ];

    for cmd in &cmds {
        recv.add_cmd(cmd);
    }

    cmds.iter().zip(recv.into_iter()).for_each(|(cmd, cmdrecv)|
        assert_eq!(cmd, &cmdrecv)
    );
}


#[test]
fn newpulse() {

    let cmd = Rc6Command::new(70, 20);
    let mut b = [0u16; 96];
    let mut len = 0;
    cmd.to_pulsetrain(&mut b, &mut len);

    let mut edge = false;
    let mut recv: EventReceiver<Rc6> = EventReceiver::new(1_000_000);

    for dist in &b[0..len] {
        edge = !edge;

        let s0 = recv.sm.state;
        let cmd = recv.edge_event(edge, *dist as u32);

        println!(
            "{} ({}): {:?} -> {:?}",
            edge as u32, dist, s0, recv.sm.state
        );

        //TODO: Fix
        if let Ok(Some(cmd)) = cmd {
            println!("cmd: {:?}", cmd);
            assert_eq!(cmd.addr, 70);
            assert_eq!(cmd.cmd, 20);
        }
    }

}


#[test]
fn basic() {
    let dists = [
        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,

        0, 106, 35, 17, 35, 17, 17, 17, 17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 35, 35, 35,

        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
    ];

    let recv: BufferReceiver<Rc6> = BufferReceiver::new(&dists, 40_000);

    let cmds = recv.iter().collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 3);

    for cmd in &cmds {
        assert_eq!(cmd.addr, 70);
        assert_eq!(cmd.cmd, 2);
    }

    /*
    let mut edge = false;

    for dist in dists.iter() {
        edge = !edge;

        let s0 = recv.sm.state;
        let cmd = recv.edge_event(edge, *dist as u32);

        println!(
            "{} ({}): {:?} -> {:?}",
            edge as u32, dist, s0, recv.sm.state
        );

        if let Ok(Some(cmd)) = cmd {
            println!("cmd: {:?}", cmd);
            assert_eq!(cmd.addr, 70);
            assert_eq!(cmd.cmd, 2);
        }
    }

     */
}
