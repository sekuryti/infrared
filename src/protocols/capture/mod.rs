use crate::{
    Command,
    ProtocolId,
    recv::{State, Receiver}
};

const BUF_LEN: usize = 128;

/// Receiver that doesn't do any decoding of the incoming signal
/// Instead it saves the distance between the edges for later processing
pub struct Capture {
    /// Samplerate
    pub samplerate: u32,
    /// Saved edges
    pub edges: [u16; BUF_LEN],
    /// Number of edges in edges
    pub n_edges: usize,
    /// Prev pin value
    pub prev_pinval: bool,
    /// Samplenum with pin change
    pub prev_samplenum: u32,
    /// Our state
    pub state: State<()>,
}


impl Command for () {
    type Addr = ();
    type Data = ();

    fn construct(_addr: (), _cmd: ()) -> Self {
    }

    fn address(&self) -> () {
    }

    fn data(&self) -> () {
    }
}

impl Receiver for Capture {
    const ID: ProtocolId = ProtocolId::Logging;
    type Cmd = ();

    fn with_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate)
    }

    fn event(&mut self, rising: bool, t: u32) -> State<Self::Cmd> {
        if !self.ready() {
            return self.state;
        }

        let t_delta = self.delta(t);

        self.state = State::Receiving;
        self.prev_samplenum = t;
        self.prev_pinval = rising;

        self.edges[self.n_edges] = t_delta;
        self.n_edges += 1;

        if self.n_edges == BUF_LEN {
            self.state = State::Done(());
        }

        self.state
    }

    fn reset(&mut self) {
        self.state = State::Idle;
        self.prev_samplenum = 0;
        self.prev_pinval = false;
        self.n_edges = 0;

        for i in 0..self.edges.len() {
            self.edges[i] = 0;
        }
    }
}

impl Capture {
    pub const fn new(samplerate: u32) -> Self {
        Self {
            edges: [0; BUF_LEN],
            samplerate,
            prev_pinval: false,
            prev_samplenum: 0,
            n_edges: 0,
            state: State::Receiving,
        }
    }

    fn ready(&self) -> bool {
        !(self.state == State::Done(()))
    }

    pub fn delta(&self, ts: u32) -> u16 {
        if self.prev_samplenum == 0 {
            return 0;
        }

        ts.wrapping_sub(self.prev_samplenum) as u16
    }

    pub fn edges(&self) -> &[u16] {
        &self.edges[0..self.n_edges]
    }
}
