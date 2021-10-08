//! This module contains Sonatine IR instructions definitions.

use std::collections::{HashMap, HashSet};

use id_arena::{Arena, Id};

use super::{Insn, InsnData, Type, Value, ValueData};

#[derive(Default, Debug, Clone)]
pub struct DataFlowGraph {
    blocks: Arena<BlockData>,
    insns: Arena<InsnData>,
    values: Arena<ValueData>,
    insn_results: HashMap<Insn, Value>,
}

impl DataFlowGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn make_block(&mut self) -> Block {
        self.blocks.alloc(BlockData::default())
    }

    pub fn make_insn(&mut self, insn: InsnData) -> Insn {
        self.insns.alloc(insn)
    }

    pub fn make_result(&mut self, insn: Insn) -> Value {
        todo!()
    }

    pub fn insn_data(&self, insn: Insn) -> &InsnData {
        &self.insns[insn]
    }

    pub fn block_data(&self, block: Block) -> &BlockData {
        &self.blocks[block]
    }

    pub fn append_block_param(&mut self, block: Block, ty: Type) -> Value {
        let value = self.values.alloc(ValueData::Param { block, ty });
        self.blocks[block].params.insert(value);
        value
    }

    pub fn remove_block_param(&mut self, block: Block, param: Value) {
        self.blocks[block].params.remove(&param);
    }

    pub fn value_def(&self, value: Value) -> ValueDef {
        match self.values[value] {
            ValueData::Insn { insn, .. } => ValueDef::Insn(insn),
            ValueData::Param { block, .. } => ValueDef::Param(block),
        }
    }

    pub fn value_ty(&self, value: Value) -> &Type {
        match &self.values[value] {
            ValueData::Insn { ty, .. } | ValueData::Param { ty, .. } => ty,
        }
    }

    pub fn insn_args(&self, insn: Insn) -> &[Value] {
        self.insn_data(insn).args()
    }

    pub fn insn_result(&self, insn: Insn) -> Option<Value> {
        self.insn_results.get(&insn).copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueDef {
    Insn(Insn),
    Param(Block),
}

/// An opaque reference to [`BlockData`]
pub type Block = Id<BlockData>;

/// A block data definition.
/// A Block data doesn't hold any information for layout of a program. It is managed by
/// [`super::layout::Layout`].
#[derive(Debug, Default, Clone)]
pub struct BlockData {
    params: HashSet<Value>,
}
