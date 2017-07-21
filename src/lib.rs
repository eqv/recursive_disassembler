extern crate capstone;
extern crate regex;
#[macro_use] extern crate lazy_static;
use std::collections::HashSet;
use std::collections::HashMap;
use capstone::{Capstone, Insn};
use regex::Regex;

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

impl Terminator{
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

    pub fn from_call(ins: &Insn) -> Self {
        if let Some(target) = Terminator::is_direct(ins){ return Terminator::Call(target)}
        return Terminator::IndCall(vec!());
    }
    pub fn from_jmp(ins: &Insn)  -> Self {
        if let Some(target) = Terminator::is_direct(ins){ return Terminator::Jump(target)}
        return Terminator::IndJump(vec!());
    }
    pub fn from_cjmp(ins: &Insn) -> Self {
        if let Some(target) = Terminator::is_direct(ins){ return Terminator::CondJump(target)}
        unreachable!();
    }
    pub fn from_ret(_: &Insn)  -> Self {
        return Terminator::Ret(vec!());
    }

}

#[derive(Debug, Eq, PartialEq)]
pub struct BasicBlock{
    pub addr: u64,
    pub size: usize,
    pub term: Terminator,
}

impl BasicBlock {
    pub fn new(addr: u64) -> Self {
        return BasicBlock{addr, size: 0, term: Terminator::Illegal };
    }

    pub fn successors(&self) -> Vec<u64>{
        match self.term {
            Terminator::Jump(ref addr) => return vec!(*addr),
            Terminator::IndJump(ref addrs) => return addrs.clone(),
            Terminator::CondJump(ref addr) => return vec!(*addr, self.addr + self.size as u64),
            Terminator::Call(ref addr) => return vec!(*addr, self.addr+self.size as u64),
            Terminator::IndCall(ref addrs) => { 
                let mut res = addrs.clone(); 
                res.push(self.addr + self.size as u64); 
                return res },
            Terminator::Ret(ref addrs) => return addrs.clone(),
            Terminator::Illegal => return vec!(),
        }
    }
}

pub struct RecursiveDisassembler{
    pub offset: u64,
    pub data: Vec<u8>,
    pub roots: HashSet<u64>,
    pub bbs: HashMap<u64, BasicBlock>,
    cs: Capstone,
    unprocessed_roots: HashSet<u64>,
}

impl RecursiveDisassembler{
    pub fn new(data: Vec<u8>, offset: u64) -> Self{
        let mut info_bitmap = Vec::with_capacity(data.len());
        info_bitmap.resize(data.len(), 0);
        let cs = Capstone::new(capstone::CsArch::ARCH_X86, capstone::CsMode::MODE_64).unwrap();
        let unprocessed_roots = HashSet::new();
        let roots = HashSet::new();
        let bbs = HashMap::new();
        return RecursiveDisassembler{offset, data, cs, roots, unprocessed_roots, bbs};
    }

    pub fn add_root(&mut self, addr: u64){
        if ! self.roots.contains(&addr) {
            self.unprocessed_roots.insert(addr);
        }
    }

    pub fn disassemble(&mut self){
        while let Some(&root) = self.unprocessed_roots.iter().next() {
            self.unprocessed_roots.remove(&root);
            if !self.in_range(root) {continue;}
            self.roots.insert(root);
            let bb = self.disassemble_bb_at(root);
            self.bbs.insert(root, bb);
        }
    }

    pub fn in_range(&self, addr: u64) -> bool {
        return ( self.offset <= addr ) && ( self.offset+self.data.len() as u64 > addr );
    }

    pub fn get_range(&self, from: u64, len:usize) -> &[u8] {
        let min = from as usize - self.offset as usize;
        assert!(from >= self.offset);
        let mut max = min + len;
        if max > self.data.len(){ 
            max = self.data.len(); 
        }
        return &self.data[min..max];
    }

    pub fn is_terminator(&self, ins: &capstone::Insn) -> Option<Terminator> {
        return Some(match ins.mnemonic().unwrap_or("illegal") {
            "call"|"lcall" => Terminator::from_call(ins),
            "jmp"|"jmpf"|"ljmp" => Terminator::from_jmp(ins),
            "ja"|"jae"|"jb"|"jbe"|"jc"|"je"|"jg"|"jge"|"jl"|"jle"|"jna"|"jnae"|"jnb"|"jnbe"|"jnc"|"jcxz"|"jrcxz"     => Terminator::from_cjmp(ins),
            "jne"|"jng"|"jnge"|"jnl"|"jnle"|"jno"|"jnp"|"jns"|"jnz"|"jo"|"jp"|"jpe"|"jpo"|"js"|"jz"|"jecxz"  => Terminator::from_cjmp(ins),
            "loop"| "loope"| "loopz"| "loopne"| "loopnz" => Terminator::from_cjmp(ins),
            "ret" | "retn" | "retf" |"leave" => Terminator::from_ret(ins),
            "illegal" => Terminator::Illegal,
            _ => return None,
        })
    }

    pub fn disassemble_bb_at(&mut self, root: u64) -> BasicBlock{
        let mut bb = BasicBlock::new(root);
        let stepsize = 50;
        let mut cur =  root;
        while self.in_range(cur) {
            match self.cs.disasm(self.get_range(cur, stepsize), root, 0) {
                 Ok(ins) => {
                    for i in ins.iter(){
                        bb.size += i.size as usize;
                        if let Some(term) = self.is_terminator(&i) { 
                            bb.term = term;
                            for &a in bb.successors().iter() { self.add_root(a) };
                            return bb; 
                        }
                    }
                    cur = root + bb.size as u64;
                 },
                 Err(_) => { return bb; }
            }
        }
        return bb;
    }
}

#[cfg(test)]
mod tests {
    use ::RecursiveDisassembler;
    use ::BasicBlock;
    use ::Terminator;
    use std::collections::HashMap;
    #[test]
    fn it_works() {
        let data = vec!(0x66, 0x40, 0x66, 0x50, 0x75, 0xfa, 0x75, 0x06, 0x66, 0x53, 0xeb, 0xfc, 0x0f, 0x04, 0xc3, 0x66, 0x83, 0xc0, 0x01); //see test.asm
        let mut disasm = RecursiveDisassembler::new(data, 0);
        disasm.add_root(0);
        disasm.disassemble();
        print!("{:?}", disasm.bbs);
        let bb1 = BasicBlock{addr: 0, size: 6, term: Terminator::CondJump(0)};
        let bb2 = BasicBlock{addr: 6, size: 2, term: Terminator::CondJump(0xe)};
        let bb3 = BasicBlock{addr: 8, size: 4, term: Terminator::Jump(8)};
        let bb4 = BasicBlock{addr: 0xe, size: 1, term: Terminator::Ret(vec!())};
        let mut sol = HashMap::new();
        sol.insert(bb1.addr, bb1);
        sol.insert(bb2.addr, bb2);
        sol.insert(bb3.addr, bb3);
        sol.insert(bb4.addr, bb4);
        assert_eq!(disasm.bbs.len(), 4);
        assert_eq!(sol[&0], disasm.bbs[&0]);
        assert_eq!(sol[&6], disasm.bbs[&6]);
        assert_eq!(sol[&8], disasm.bbs[&8]);
        assert_eq!(sol[&0xe], disasm.bbs[&0xe]);
    }
}
