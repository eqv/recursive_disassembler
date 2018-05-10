use terminator::Terminator;

#[derive(Debug, Eq, PartialEq)]
pub struct BasicBlock {
    pub addr: u64,
    pub size: usize,
    pub term: Terminator,
}

impl BasicBlock {
    pub fn new(addr: u64) -> Self {
        return BasicBlock { addr, size: 0, term: Terminator::Illegal };
    }

    pub fn successors(&self) -> Vec<u64> {
        match self.term {
            Terminator::Jump(ref addr) => return vec!(*addr),
            Terminator::IndJump(ref addrs) => return addrs.clone(),
            Terminator::CondJump(ref addr) => return vec!(*addr, self.addr + self.size as u64),
            Terminator::Call(ref addr) => return vec!(*addr, self.addr + self.size as u64),
            Terminator::IndCall(ref addrs) => {
                let mut res = addrs.clone();
                res.push(self.addr + self.size as u64);
                return res; }
            Terminator::Ret(ref addrs) => return addrs.clone(),
            Terminator::Illegal => return vec!(),
        }
    }
}