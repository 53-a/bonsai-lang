// SPDX-License-Identifier: Unlicense
use crate::{ast, ir};
use anyhow::{anyhow, Result};

pub struct IrGen {
    ast_arena: ast::Arena,
    ir_arena: ir::Arena,
}

impl IrGen {
    fn new(ast_arena: ast::Arena) -> Self {
        Self {
            ast_arena,
            ir_arena: ir::Arena::new(),
        }
    }

    fn new_node(&mut self, kind: ir::Kind) -> ir::Id {
        self.ir_arena.alloc(ir::Node { kind })
    }

    fn map_biop_kind(kind: &ast::BiOpKind) -> Result<ir::OpKind> {
        match kind {
            ast::BiOpKind::Add => Ok(ir::OpKind::IAdd),
            ast::BiOpKind::Sub => Ok(ir::OpKind::ISub),
            ast::BiOpKind::Mul => Ok(ir::OpKind::IMul),
            ast::BiOpKind::Div => Ok(ir::OpKind::IDiv),
        }
    }

    fn generate_impl(&mut self, root: ast::Id) -> Result<ir::Id> {
        let kind = &self
            .ast_arena
            .get(root)
            .ok_or(anyhow!("failed to get ast node from arena"))?
            .kind
            .clone();
        match kind {
            ast::NodeKind::Lit(lit) => match lit {
                &ast::LitKind::IntLit(i) => Ok(self.new_node(ir::Kind::IntValue(i))),
            },
            ast::NodeKind::Paren(e) => self.generate_impl(*e),
            ast::NodeKind::BiOp(kind, lhs, rhs) => {
                let op_kind = Self::map_biop_kind(&kind)?;
                let lhs = self.generate_impl(*lhs)?;
                let rhs = self.generate_impl(*rhs)?;
                let args = vec![lhs, rhs];
                Ok(self.new_node(ir::Kind::Op(op_kind, args)))
            }
        }
    }
}

pub fn generate(ast_arena: ast::Arena, root: ast::Id) -> Result<(ir::Arena, ir::Id)> {
    let mut irgen = IrGen::new(ast_arena);
    let ir = irgen.generate_impl(root)?;
    Ok((irgen.ir_arena, ir))
}
