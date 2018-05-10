use std::collections::HashSet;
use std::collections::HashMap;
use bb_disassembler::BBDisassembler;
use basic_block::BasicBlock;

pub struct RecursiveDisassembler<T: BBDisassembler> {
    pub offset: u64,
    pub data: Vec<u8>,
    pub roots: HashSet<u64>,
    pub bbs: HashMap<u64, BasicBlock>,
    dis: T,
    unprocessed_roots: HashSet<u64>,
}

impl<T: BBDisassembler> RecursiveDisassembler<T> {
    pub fn new(data: Vec<u8>, offset: u64, dis: T) -> Self {
        let unprocessed_roots = HashSet::new();
        let roots = HashSet::new();
        let bbs = HashMap::new();
        return RecursiveDisassembler { offset, data, dis, roots, unprocessed_roots, bbs };
    }

    pub fn add_root(&mut self, addr: u64) {
        if !self.roots.contains(&addr) {
            self.unprocessed_roots.insert(addr);
        }
    }

    pub fn disassemble(&mut self) {
        while let Some(&root) = self.unprocessed_roots.iter().next() {
            self.unprocessed_roots.remove(&root);
            if !self.in_range(root) { continue; }
            self.roots.insert(root);
            let bb = self.dis.get_basic_block(root, self.offset, &self.data);

            for addr in bb.successors() {
                self.add_root(addr);
            }
            self.bbs.insert(root, bb);
        }
    }

    fn in_range(&self, addr: u64) -> bool {
        return (self.offset <= addr) && (self.offset + self.data.len() as u64 > addr);
    }
}