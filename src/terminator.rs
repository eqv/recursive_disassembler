#[derive(Debug, Eq, PartialEq)]
pub enum Terminator {
    Jump(u64),
    IndJump(Vec<u64>),
    CondJump(u64),
    Call(u64),
    IndCall(Vec<u64>),
    Ret(Vec<u64>),
    Illegal,
}

