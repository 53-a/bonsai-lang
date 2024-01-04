// SPDX-License-Identifier: Unlicense
#[derive(Debug, Clone, PartialEq)]
pub enum OpKind {
    IAdd,
    ISub,
    IMul,
    IDiv,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    IntValue(i64),
    Op(OpKind, Vec<Id>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: Kind,
}

pub type Id = id_arena::Id<Node>;
pub type Arena = id_arena::Arena<Node>;
