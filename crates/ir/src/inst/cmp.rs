use macros::Inst;

use crate::{impl_ir_write, ValueId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Lt {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Lt);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Gt {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Gt);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Slt {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Slt);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Sgt {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Sgt);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Le {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Le);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Ge {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Ge);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Sle {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Sle);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Sge {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Sge);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Eq {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Eq);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct Ne {
    #[inst(value)]
    lhs: ValueId,
    #[inst(value)]
    rhs: ValueId,
}
impl_ir_write!(Ne);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inst)]
pub struct IsZero {
    #[inst(value)]
    lhs: ValueId,
}
impl_ir_write!(IsZero);
