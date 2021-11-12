//! This module contains Sonatine IR instructions definitions.

// TODO: Add type checker for instruction arguments.
use std::fmt;

use smallvec::SmallVec;

use super::{types::U256, Block, DataFlowGraph, Type, Value};

/// An opaque reference to [`InsnData`]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
pub struct Insn(pub u32);
cranelift_entity::entity_impl!(Insn);

/// An instruction data definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsnData {
    /// Immediate instruction.
    Immediate { code: ImmediateOp },

    /// Binary instruction.
    Binary { code: BinaryOp, args: [Value; 2] },

    /// Cast operations.
    Cast {
        code: CastOp,
        args: [Value; 1],
        ty: Type,
    },

    /// Load a value from memory.
    Load { args: [Value; 1], ty: Type },

    /// Store a value to memory.
    Store { args: [Value; 2] },

    /// Unconditional jump operaitons.
    Jump { code: JumpOp, dests: [Block; 1] },

    /// Conditional jump operations.
    Branch { args: [Value; 1], dests: [Block; 2] },

    /// Return.
    Return { args: SmallVec<[Value; 8]> },

    /// Phi funcion.
    Phi {
        values: SmallVec<[Value; 8]>,
        blocks: SmallVec<[Block; 8]>,
        ty: Type,
    },
}

impl InsnData {
    pub fn jump(dest: Block) -> InsnData {
        InsnData::Jump {
            code: JumpOp::Jump,
            dests: [dest],
        }
    }

    pub fn branch_dests(&self) -> &[Block] {
        match self {
            Self::Jump { dests, .. } => dests.as_ref(),
            Self::Branch { dests, .. } => dests.as_ref(),
            _ => &[],
        }
    }

    pub fn branch_dests_mut(&mut self) -> &mut [Block] {
        match self {
            Self::Jump { dests, .. } => dests.as_mut(),
            Self::Branch { dests, .. } => dests.as_mut(),
            _ => &mut [],
        }
    }

    pub fn args(&self) -> &[Value] {
        match self {
            Self::Binary { args, .. } | Self::Store { args, .. } => args,
            Self::Cast { args, .. } | Self::Load { args, .. } | Self::Branch { args, .. } => args,
            Self::Phi { values: args, .. } | Self::Return { args } => args,
            _ => &[],
        }
    }

    pub fn args_mut(&mut self) -> &mut [Value] {
        match self {
            Self::Binary { args, .. } | Self::Store { args, .. } => args,
            Self::Cast { args, .. } | Self::Load { args, .. } | Self::Branch { args, .. } => args,
            Self::Phi { values, .. } => values,
            _ => &mut [],
        }
    }

    pub fn replace_arg(&mut self, new_arg: Value, idx: usize) {
        self.args_mut()[idx] = new_arg;
    }

    pub fn has_side_effect(&self) -> bool {
        // We assume `Load` has side effect because it may cause trap.
        matches!(
            self,
            InsnData::Load { .. } | InsnData::Store { .. } | InsnData::Return { .. }
        )
    }

    pub(super) fn result_type(&self, dfg: &DataFlowGraph) -> Option<Type> {
        match self {
            Self::Immediate { code } => Some(code.result_type()),
            Self::Binary { code, args } => Some(code.result_type(dfg, args)),
            Self::Cast { ty, .. } | Self::Load { ty, .. } => Some(ty.clone()),
            Self::Phi { ty, .. } => Some(ty.clone()),
            _ => None,
        }
    }
}

/// Immidiates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmediateOp {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
}

impl ImmediateOp {
    fn result_type(&self) -> Type {
        match self {
            Self::I8(_) | Self::U8(_) => Type::I8,
            Self::I16(_) | Self::U16(_) => Type::I16,
            Self::I32(_) | Self::U32(_) => Type::I32,
            Self::I64(_) | Self::U64(_) => Type::I64,
            Self::I128(_) | Self::U128(_) => Type::I128,
            Self::U256(_) => Type::I256,
        }
    }
}

impl fmt::Display for ImmediateOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I8(value) => write!(f, "imm_i8 {}", value),
            Self::I16(value) => write!(f, "imm_i16 {}", value),
            Self::I32(value) => write!(f, "imm_i32 {}", value),
            Self::I64(value) => write!(f, "imm_i64 {}", value),
            Self::I128(value) => write!(f, "imm_i128 {}", value),
            Self::U8(value) => write!(f, "imm_u8 {}", value),
            Self::U16(value) => write!(f, "imm_u16 {}", value),
            Self::U32(value) => write!(f, "imm_u32 {}", value),
            Self::U64(value) => write!(f, "imm_u64 {}", value),
            Self::U128(value) => write!(f, "imm_u128 {}", value),
            Self::U256(value) => write!(f, "imm_u256 {}", value),
        }
    }
}

/// Binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    UDiv,
    SDiv,
    Lt,
    Gt,
    Slt,
    Sgt,
    Eq,
    And,
    Or,
}

impl BinaryOp {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::UDiv => "udiv",
            Self::SDiv => "sdiv",
            Self::Lt => "lt",
            Self::Gt => "gt",
            Self::Slt => "slt",
            Self::Sgt => "sgt",
            Self::Eq => "eq",
            Self::And => "and",
            Self::Or => "or",
        }
    }

    fn result_type(self, dfg: &DataFlowGraph, args: &[Value; 2]) -> Type {
        dfg.value_ty(args[0]).clone()
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastOp {
    Sext,
    Zext,
    Trunc,
}

impl CastOp {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Sext => "sext",
            Self::Zext => "zext",
            Self::Trunc => "trunc",
        }
    }
}

impl fmt::Display for CastOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unconditional jump operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpOp {
    Jump,
    FallThrough,
}

impl JumpOp {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Jump => "jump",
            Self::FallThrough => "fallthrough",
        }
    }
}

impl fmt::Display for JumpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
