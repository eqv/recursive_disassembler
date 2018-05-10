use capstone::{Capstone, Insn, CsArch, CsMode};
use regex::Regex;
use basic_block::BasicBlock;
use terminator::Terminator;

pub trait BBDisassembler {
    fn get_basic_block(&mut self, addr: u64, mapping_offset: u64, mapped_data: &[u8]) -> BasicBlock;
}

pub struct BBDisasmCapstoneX86 {
    cap: Capstone
}

impl BBDisasmCapstoneX86{
    pub fn new_32() -> Self{
        return BBDisasmCapstoneX86{cap: Capstone::new(CsArch::ARCH_X86, CsMode::MODE_32).unwrap()};
    }
    pub fn new_64() -> Self{
        return BBDisasmCapstoneX86{cap: Capstone::new(CsArch::ARCH_X86, CsMode::MODE_64).unwrap()};
    }
}

impl BBDisassembler for BBDisasmCapstoneX86 {
    fn get_basic_block(&mut self, addr: u64, mapping_offset: u64, mapped_data: &[u8]) -> BasicBlock {
        let mut bb = BasicBlock::new(addr);
        let stepsize = 50;
        let mut cur = addr;
        while mapping_offset <= cur && (cur - mapping_offset) < mapped_data.len() as u64 {
            match self.cap.disasm(get_range(cur, stepsize, mapping_offset, mapped_data), addr, 0) {
                Ok(ins) => {
                    for i in ins.iter() {
                        bb.size += i.size as usize;
                        if let Some(term) = is_terminator(&i) {
                            bb.term = term;
                            return bb;
                        }
                    }
                    cur = addr + bb.size as u64;
                }
                Err(_) => { return bb; }
            }
        }
        return bb;
    }
}

fn is_direct(ins: &Insn) -> Option<u64> {
    lazy_static! {
        static ref DIRECT_RE: Regex = Regex::new(r"\A(0x)?([a-fA-F0-9]+)\z").unwrap();
    }
    if let Some(cap) = DIRECT_RE.captures(ins.op_str().unwrap_or("invalid")) {
        let addr = u64::from_str_radix(&cap[2], 16).unwrap();
        return Some(addr);
    }
    return None;
}

fn from_call(ins: &Insn) -> Terminator {
    if let Some(target) = is_direct(ins) { return Terminator::Call(target); }
    return Terminator::IndCall(vec!());
}

fn from_jmp(ins: &Insn) -> Terminator {
    if let Some(target) = is_direct(ins) { return Terminator::Jump(target); }
    return Terminator::IndJump(vec!());
}

fn from_cjmp(ins: &Insn) -> Terminator {
    if let Some(target) = is_direct(ins) { return Terminator::CondJump(target); }
    unreachable!();
}

fn from_ret(_: &Insn) -> Terminator {
    return Terminator::Ret(vec!());
}

fn is_terminator(ins: &Insn) -> Option<Terminator> {
    return Some(match ins.mnemonic().unwrap_or("illegal") {
        "call" | "lcall" => from_call(ins),
        "jmp" | "jmpf" | "ljmp" => from_jmp(ins),
        "ja" | "jae" | "jb" | "jbe" | "jc" | "je" | "jg" | "jge" | "jl" | "jle" | "jna" | "jnae" | "jnb" | "jnbe" | "jnc" | "jcxz" | "jrcxz" => from_cjmp(ins),
        "jne" | "jng" | "jnge" | "jnl" | "jnle" | "jno" | "jnp" | "jns" | "jnz" | "jo" | "jp" | "jpe" | "jpo" | "js" | "jz" | "jecxz" => from_cjmp(ins),
        "loop" | "loope" | "loopz" | "loopne" | "loopnz" => from_cjmp(ins),
        "ret" | "retn" | "retf" | "leave" => from_ret(ins),
        "illegal" => Terminator::Illegal,
        _ => return None,
    });
}

fn get_range(from: u64, len: usize, mapping_offset: u64, mapped_data: &[u8]) -> &[u8] {
    let min = from as usize - mapping_offset as usize;
    assert!(from >= mapping_offset);
    let mut max = min + len;
    if max > mapped_data.len() {
        max = mapped_data.len();
    }
    return &mapped_data[min..max];
}
