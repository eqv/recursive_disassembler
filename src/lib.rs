extern crate capstone;
extern crate regex;
#[macro_use] extern crate lazy_static;

mod recursive_disassembler;
mod bb_disassembler;
mod basic_block;
mod terminator;

pub use recursive_disassembler::RecursiveDisassembler;
pub use bb_disassembler::BBDisassembler;
pub use basic_block::BasicBlock;
pub  use terminator::Terminator;

#[cfg(test)]
mod tests {
    use ::RecursiveDisassembler;
    use ::BasicBlock;
    use ::Terminator;
    use std::collections::HashMap;
    use capstone::{Capstone, CsArch, CsMode};

    #[test]
    fn it_works() {
        let data = vec!(0x66, 0x40, 0x66, 0x50, 0x75, 0xfa, 0x75, 0x06, 0x66, 0x53, 0xeb, 0xfc, 0x0f, 0x04, 0xc3, 0x66, 0x83, 0xc0, 0x01); //see test.asm
        let cs = Capstone::new(CsArch::ARCH_X86, CsMode::MODE_32).unwrap();
        let mut disasm = RecursiveDisassembler::new(data, 0, cs);
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
