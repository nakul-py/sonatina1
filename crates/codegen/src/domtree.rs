//! This module contains dominantor tree related structs.
//!
//! The algorithm is based on Keith D. Cooper., Timothy J. Harvey., and Ken
//! Kennedy.: A Simple, Fast Dominance Algorithm: <https://www.cs.rice.edu/~keith/EMBED/dom.pdf>

use std::collections::BTreeSet;

use cranelift_entity::{packed_option::PackedOption, SecondaryMap};
use sonatina_ir::{BlockId, ControlFlowGraph};

#[derive(Default, Debug)]
pub struct DomTree {
    doms: SecondaryMap<BlockId, PackedOption<BlockId>>,
    rpo: Vec<BlockId>,
}

impl DomTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.doms.clear();
        self.rpo.clear();
    }

    /// Returns the immediate dominator of the `block`.
    /// Returns None if the `block` is unreachable from the entry block, or the
    /// `block` is the entry block itself.
    pub fn idom_of(&self, block: BlockId) -> Option<BlockId> {
        if self.rpo[0] == block {
            return None;
        }
        self.doms[block].expand()
    }

    /// Returns `true` if block1 strictly dominates block2.
    pub fn strictly_dominates(&self, block1: BlockId, block2: BlockId) -> bool {
        let mut current_block = block2;
        while let Some(block) = self.idom_of(current_block) {
            if block == block1 {
                return true;
            }
            current_block = block;
        }

        false
    }

    /// Returns `true` if block1 dominates block2.
    pub fn dominates(&self, block1: BlockId, block2: BlockId) -> bool {
        if block1 == block2 {
            return true;
        }

        self.strictly_dominates(block1, block2)
    }

    pub fn compute(&mut self, cfg: &ControlFlowGraph) {
        self.clear();

        self.rpo = cfg.post_order().collect();
        self.rpo.reverse();

        let block_num = self.rpo.len();

        if self.doms.capacity() < block_num {
            self.doms = SecondaryMap::with_capacity(block_num);
        } else {
            self.doms.clear();
        }

        let mut rpo_nums = SecondaryMap::with_capacity(block_num);
        for (i, &block) in self.rpo.iter().enumerate() {
            rpo_nums[block] = (block_num - i) as u32;
        }

        match self.rpo.first() {
            Some(&entry) => self.doms[entry] = entry.into(),
            None => return,
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block in self.rpo.iter().skip(1) {
                let processed_pred =
                    match cfg.preds_of(block).find(|&&pred| self.doms[pred].is_some()) {
                        Some(pred) => *pred,
                        _ => continue,
                    };
                let mut new_dom = processed_pred;

                for &pred in cfg.preds_of(block) {
                    if pred != processed_pred && self.doms[pred].is_some() {
                        new_dom = self.intersect(new_dom, pred, &rpo_nums);
                    }
                }
                if Some(new_dom) != self.doms[block].expand() {
                    changed = true;
                    self.doms[block] = new_dom.into();
                }
            }
        }
    }

    /// Compute dominance frontiers of each blocks.
    pub fn compute_df(&self, cfg: &ControlFlowGraph) -> DFSet {
        let mut df = DFSet::default();

        for &block in &self.rpo {
            if cfg.pred_num_of(block) < 2 {
                continue;
            }
            for pred in cfg.preds_of(block) {
                let mut runner = *pred;
                while PackedOption::from(runner) != self.doms[block] && self.is_reachable(runner) {
                    df.0[runner].insert(block);
                    runner = self.doms[runner].unwrap();
                }
            }
        }

        df
    }

    /// Returns `true` if block is reachable from the entry block.
    pub fn is_reachable(&self, block: BlockId) -> bool {
        self.idom_of(block).is_some()
    }

    /// Returns blocks in RPO.
    pub fn rpo(&self) -> &[BlockId] {
        &self.rpo
    }

    fn intersect(
        &self,
        mut b1: BlockId,
        mut b2: BlockId,
        rpo_nums: &SecondaryMap<BlockId, u32>,
    ) -> BlockId {
        while b1 != b2 {
            while rpo_nums[b1] < rpo_nums[b2] {
                b1 = self.doms[b1].unwrap();
            }
            while rpo_nums[b2] < rpo_nums[b1] {
                b2 = self.doms[b2].unwrap();
            }
        }

        b1
    }
}

/// Dominance frontiers of each blocks.
#[derive(Default, Debug)]
pub struct DFSet(SecondaryMap<BlockId, BTreeSet<BlockId>>);

impl DFSet {
    pub fn frontiers(&self, block: BlockId) -> impl Iterator<Item = &BlockId> {
        self.0[block].iter()
    }

    pub fn in_frontier_of(&self, block: BlockId, of: BlockId) -> bool {
        self.0[of].contains(&block)
    }

    pub fn frontier_num_of(&self, of: BlockId) -> usize {
        self.0[of].len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

#[derive(Default)]
pub struct DominatorTreeTraversable {
    children: SecondaryMap<BlockId, Vec<BlockId>>,
}

impl DominatorTreeTraversable {
    pub fn compute(&mut self, domtree: &DomTree) {
        for &block in domtree.rpo() {
            if let Some(idom) = domtree.idom_of(block) {
                self.children[idom].push(block)
            }
        }
    }

    pub fn children_of(&self, block: BlockId) -> &[BlockId] {
        &self.children[block]
    }

    pub fn clear(&mut self) {
        self.children.clear();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::many_single_char_names)]

    use sonatina_ir::{
        builder::test_util::*,
        inst::control_flow::{Br, BrTable, Jump, Return},
        prelude::*,
        Function, Type,
    };

    use super::*;

    fn calc_dom(func: &Function) -> (DomTree, DFSet) {
        let mut cfg = ControlFlowGraph::default();
        cfg.compute(func);
        let mut dom_tree = DomTree::default();
        dom_tree.compute(&cfg);
        let df = dom_tree.compute_df(&cfg);
        (dom_tree, df)
    }

    fn test_df(df: &DFSet, of: BlockId, frontiers: &[BlockId]) -> bool {
        if df.frontier_num_of(of) != frontiers.len() {
            return false;
        }

        for &block in frontiers {
            if !df.in_frontier_of(block, of) {
                return false;
            }
        }
        true
    }

    #[test]
    fn dom_tree_if_else() {
        let mb = test_module_builder();
        let (evm, mut builder) = test_func_builder(&mb, &[], Type::Unit);
        let is = evm.inst_set();

        let entry_block = builder.append_block();
        let then_block = builder.append_block();
        let else_block = builder.append_block();
        let merge_block = builder.append_block();

        builder.switch_to_block(entry_block);
        let v0 = builder.make_imm_value(true);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, else_block, then_block));

        builder.switch_to_block(then_block);
        builder.insert_inst_no_result_with(|| Jump::new(is, merge_block));

        builder.switch_to_block(else_block);
        builder.insert_inst_no_result_with(|| Jump::new(is, merge_block));

        builder.switch_to_block(merge_block);
        builder.insert_inst_no_result_with(|| Return::new(is, None));

        builder.seal_all();
        builder.finish();

        let module = mb.build();
        let func_ref = module.funcs()[0];
        let (dom_tree, df) = module.func_store.view(func_ref, calc_dom);

        assert_eq!(dom_tree.idom_of(entry_block), None);
        assert_eq!(dom_tree.idom_of(then_block), Some(entry_block));
        assert_eq!(dom_tree.idom_of(else_block), Some(entry_block));
        assert_eq!(dom_tree.idom_of(merge_block), Some(entry_block));

        assert!(test_df(&df, entry_block, &[]));
        assert!(test_df(&df, then_block, &[merge_block]));
        assert!(test_df(&df, else_block, &[merge_block]));
        assert!(test_df(&df, merge_block, &[]));
    }

    #[test]
    fn unreachable_edge() {
        let mb = test_module_builder();
        let (evm, mut builder) = test_func_builder(&mb, &[], Type::Unit);
        let is = evm.inst_set();

        let a = builder.append_block();
        let b = builder.append_block();
        let c = builder.append_block();
        let d = builder.append_block();
        let e = builder.append_block();

        builder.switch_to_block(a);
        let v0 = builder.make_imm_value(true);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, b, c));

        builder.switch_to_block(b);
        builder.insert_inst_no_result_with(|| Jump::new(is, e));

        builder.switch_to_block(c);
        builder.insert_inst_no_result_with(|| Jump::new(is, e));

        builder.switch_to_block(d);
        builder.insert_inst_no_result_with(|| Jump::new(is, e));

        builder.switch_to_block(e);
        builder.insert_inst_no_result_with(|| Return::new(is, None));

        builder.seal_all();
        builder.finish();

        let module = mb.build();
        let func_ref = module.funcs()[0];
        let (dom_tree, df) = module.func_store.view(func_ref, calc_dom);

        assert_eq!(dom_tree.idom_of(a), None);
        assert_eq!(dom_tree.idom_of(b), Some(a));
        assert_eq!(dom_tree.idom_of(c), Some(a));
        assert_eq!(dom_tree.idom_of(d), None);
        assert!(!dom_tree.is_reachable(d));
        assert_eq!(dom_tree.idom_of(e), Some(a));

        assert!(test_df(&df, a, &[]));
        assert!(test_df(&df, b, &[e]));
        assert!(test_df(&df, c, &[e]));
        assert!(test_df(&df, d, &[]));
        assert!(test_df(&df, e, &[]));
    }

    #[test]
    fn dom_tree_complex() {
        let mb = test_module_builder();
        let (evm, mut builder) = test_func_builder(&mb, &[], Type::Unit);
        let is = evm.inst_set();

        let a = builder.append_block();
        let b = builder.append_block();
        let c = builder.append_block();
        let d = builder.append_block();
        let e = builder.append_block();
        let f = builder.append_block();
        let g = builder.append_block();
        let h = builder.append_block();
        let i = builder.append_block();
        let j = builder.append_block();
        let k = builder.append_block();
        let l = builder.append_block();
        let m = builder.append_block();

        builder.switch_to_block(a);
        let v0 = builder.make_imm_value(true);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, c, b));

        builder.switch_to_block(b);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, g, d));

        builder.switch_to_block(c);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, h, e));

        builder.switch_to_block(d);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, g, f));

        builder.switch_to_block(e);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, h, c));

        builder.switch_to_block(f);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, k, i));

        builder.switch_to_block(g);
        builder.insert_inst_no_result_with(|| Jump::new(is, j));

        builder.switch_to_block(h);
        builder.insert_inst_no_result_with(|| Jump::new(is, m));

        builder.switch_to_block(i);
        builder.insert_inst_no_result_with(|| Jump::new(is, l));

        builder.switch_to_block(j);
        builder.insert_inst_no_result_with(|| Jump::new(is, i));

        builder.switch_to_block(k);
        builder.insert_inst_no_result_with(|| Jump::new(is, l));

        builder.switch_to_block(l);
        builder.insert_inst_no_result_with(|| Br::new(is, v0, m, b));

        builder.switch_to_block(m);
        builder.insert_inst_no_result_with(|| Return::new(is, None));

        builder.seal_all();
        builder.finish();

        let module = mb.build();
        let func_ref = module.funcs()[0];
        let (dom_tree, df) = module.func_store.view(func_ref, calc_dom);

        assert_eq!(dom_tree.idom_of(a), None);
        assert_eq!(dom_tree.idom_of(b), Some(a));
        assert_eq!(dom_tree.idom_of(c), Some(a));
        assert_eq!(dom_tree.idom_of(d), Some(b));
        assert_eq!(dom_tree.idom_of(e), Some(c));
        assert_eq!(dom_tree.idom_of(f), Some(d));
        assert_eq!(dom_tree.idom_of(g), Some(b));
        assert_eq!(dom_tree.idom_of(h), Some(c));
        assert_eq!(dom_tree.idom_of(i), Some(b));
        assert_eq!(dom_tree.idom_of(j), Some(g));
        assert_eq!(dom_tree.idom_of(k), Some(f));

        assert!(test_df(&df, a, &[]));
        assert!(test_df(&df, b, &[b, m]));
        assert!(test_df(&df, c, &[c, m]));
        assert!(test_df(&df, d, &[g, i, l]));
        assert!(test_df(&df, e, &[c, h]));
        assert!(test_df(&df, f, &[i, l]));
        assert!(test_df(&df, g, &[i]));
        assert!(test_df(&df, h, &[m]));
        assert!(test_df(&df, i, &[l]));
        assert!(test_df(&df, j, &[i]));
        assert!(test_df(&df, k, &[l]));
        assert!(test_df(&df, l, &[b, m]));
        assert!(test_df(&df, m, &[]));
    }

    #[test]
    fn dom_tree_br_table() {
        let mb = test_module_builder();
        let (evm, mut builder) = test_func_builder(&mb, &[], Type::Unit);
        let is = evm.inst_set();

        let a = builder.append_block();
        let b = builder.append_block();
        let c = builder.append_block();
        let d = builder.append_block();
        let e = builder.append_block();
        let f = builder.append_block();

        builder.switch_to_block(a);
        let v0 = builder.make_imm_value(0i32);
        let v1 = builder.make_imm_value(1i32);
        let v2 = builder.make_imm_value(2i32);
        builder
            .insert_inst_no_result_with(|| BrTable::new(is, v0, Some(b), vec![(v1, c), (v2, d)]));

        builder.switch_to_block(b);
        let v3 = builder.make_imm_value(true);
        builder.insert_inst_no_result_with(|| Br::new(is, v3, a, e));

        builder.switch_to_block(c);
        builder.insert_inst_no_result_with(|| Jump::new(is, f));

        builder.switch_to_block(d);
        builder.insert_inst_no_result_with(|| Jump::new(is, f));

        builder.switch_to_block(e);
        builder.insert_inst_no_result_with(|| Return::new(is, None));

        builder.switch_to_block(f);
        builder.insert_inst_no_result_with(|| Return::new(is, None));

        builder.seal_all();
        builder.finish();

        let module = mb.build();
        let func_ref = module.funcs()[0];
        let (dom_tree, df) = module.func_store.view(func_ref, calc_dom);

        assert_eq!(dom_tree.idom_of(a), None);
        assert_eq!(dom_tree.idom_of(b), Some(a));
        assert_eq!(dom_tree.idom_of(c), Some(a));
        assert_eq!(dom_tree.idom_of(d), Some(a));
        assert_eq!(dom_tree.idom_of(e), Some(b));
        assert_eq!(dom_tree.idom_of(f), Some(a));

        assert!(test_df(&df, a, &[]));
        assert!(test_df(&df, b, &[]));
        assert!(test_df(&df, c, &[f]));
        assert!(test_df(&df, d, &[f]));
        assert!(test_df(&df, e, &[]));
        assert!(test_df(&df, f, &[]));
    }
}
