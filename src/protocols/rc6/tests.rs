#[cfg(test)]
mod tests {
    use crate::protocols::rc6::Rc6;
    use crate::recv::*;

    #[test]
    fn basic() {
        let dists = [
            0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,

            0, 106, 35, 17, 35, 17, 17, 17, 17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 35, 35, 35,

            0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
        ];

        let recv: BufferedReceiver<Rc6> = BufferedReceiver::new(&dists, 40_000);

        let cmds = recv.collect::<std::vec::Vec<_>>();

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
}
