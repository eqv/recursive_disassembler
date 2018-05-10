Recursive Disassembler
=====================
This crate implements a very simple recursive disassembler based on Capstone. Provide some data and a set of start
offsets (disassembly roots), then this crate generates a list of basicblocks, following jump instructions and calls.

Usage:
=====
simply add `recursive_disassembler="2.*"` to your `cargo.toml`.

```rust
extern crate recursive_disassembler;
use recursive_disassembler::{RecursiveDisassembler, BBDisasmCapstoneX86};

fn main() {
    let data = vec!(0x66, 0x40, 0x66, 0x50, 0x75, 0xfa, 0x75, 0x06, 0x66, 0x53, 0xeb, 0xfc, 0x0f, 0x04, 0xc3, 0x66, 0x83, 0xc0, 0x01); //see test.asm
    let bbdasm = BBDisasmCapstoneX86::new_32();
    let base = 0x40000;
    let mut disasm = RecursiveDisassembler::new(data, base, bbdasm);
    disasm.add_root(0x40000);
    disasm.disassemble();
    print!("{:#?}", disasm.bbs);
}

```

Results in the following output:
```
{
    262144: BasicBlock {
        addr: 262144,
        size: 6,
        term: CondJump(
            262144
        )
    },
    262150: BasicBlock {
        addr: 262150,
        size: 2,
        term: CondJump(
            262158
        )
    },
    262152: BasicBlock {
        addr: 262152,
        size: 4,
        term: Jump(
            262152
        )
    },
    262158: BasicBlock {
        addr: 262158,
        size: 1,
        term: Ret(
            []
        )
    }
}
```

Status
======

* Currently only supports x86 (32/64).
* Currently doesn't resolve jumptables and similar constructs
* Should probably do a linear sweep pass first to find more stuff
