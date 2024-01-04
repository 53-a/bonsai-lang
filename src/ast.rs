// SPDX-License-Identifier: Unlicense
#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    IntLit(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BiOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Lit(LitKind),
    Paren(Id),
    BiOp(BiOpKind, Id, Id),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
}

pub type Id = id_arena::Id<Node>;
pub type Arena = id_arena::Arena<Node>;
