//! This module contains Sonatine IR instructions definitions.

use std::collections::HashSet;

use id_arena::Id;
use primitive_types::U256;

use super::{Block, Value};

/// An opaque reference to [`InsnData`]
pub type Insn = Id<InsnData>;

/// An instruction data definition.
#[derive(Debug, Clone)]
pub enum InsnData {
    /// Immediate instruction.
    Immediate { code: ImmediateOp },

    /// Unary instruction.
    Unary { code: UnaryOp, args: [Value; 1] },

    /// Binary instruction.
    Binary { code: BinaryOp, args: [Value; 2] },

    /// Unconditional jump operaitons.
    Jump {
        code: JumpOp,
        dest: Block,
        /// Block paramters.
        params: HashSet<Value>,
    },

    /// Conditional jump operations.
    Branch {
        code: BranchOp,
        args: [Value; 1],
        dest: Block,
        /// Block parameters.
        params: HashSet<Value>,
    },
}

impl InsnData {
    pub fn branch_dest(&self) -> Option<Block> {
        match self {
            Self::Jump { dest, .. } => Some(*dest),
            _ => None,
        }
    }

    pub fn args(&self) -> &[Value] {
        match self {
            Self::Unary { args, .. } => args,
            Self::Binary { args, .. } => args,
            Self::Branch { args, .. } => args,
            _ => &[],
        }
    }
}

/// Immidiates.
#[derive(Debug, Clone)]
pub enum ImmediateOp {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
}

/// Unary operations.
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Sext,
    Zext,
}

/// Binary operations.
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Slt,
    Sgt,
    Eq,
    And,
    Or,
    Load,
    Store,
}

/// Unconditional jump operations.
#[derive(Debug, Clone, Copy)]
pub enum JumpOp {
    Jump,
}

/// Conditional jump operations.
#[derive(Debug, Clone, Copy)]
pub enum BranchOp {
    /// Branch if cond is zero.
    Brz,

    /// Branch if cond is non zero.
    Brnz,
}
