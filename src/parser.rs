// SPDX-License-Identifier: Unlicense
use crate::ast;
use anyhow::{anyhow, Result};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Context {
    pub arena: RefCell<ast::Arena>,
}

peg::parser! {
    grammar main_parser(context: &Context) for str {
        #[cache]
        rule _() = quiet!{[' '|'\t'|'\r'|'\n']*{}}

        rule node(r: rule<ast::NodeKind>) -> ast::Id = n: r() {
            let mut arena = context.arena.borrow_mut();
            arena.alloc(ast::Node{ kind: n })
        }

        rule int_lit() -> ast::NodeKind = _ n:$(['0' ..= '9']+) {
            ast::NodeKind::Lit(ast::LitKind::IntLit(n.parse().unwrap()))
        }

        rule expr() -> ast::Id = precedence! {
            _:position!() p:@ _:position!() {
                let mut arena = context.arena.borrow_mut();
                arena.alloc(
                    ast::Node {
                        kind: p,
                    }
                )
            }
            --
            x:(@) (_ "+") y:@ { ast::NodeKind::BiOp(ast::BiOpKind::Add, x, y) }
            x:(@) (_ "-") y:@ { ast::NodeKind::BiOp(ast::BiOpKind::Sub, x, y) }
            --
            x:(@) (_ "*") y:@ { ast::NodeKind::BiOp(ast::BiOpKind::Mul, x, y) }
            x:(@) (_ "/") y:@ { ast::NodeKind::BiOp(ast::BiOpKind::Div, x, y) }
            --
            n: int_lit() { n }

            _ "(" e:expr() _ ")" { ast::NodeKind::Paren(e) }
        }
        pub rule parse() -> ast::Id = n:expr() _ { n }
    }
}

pub fn parse(source: &str) -> Result<(ast::Arena, ast::Id)> {
    let arena_cell = RefCell::new(ast::Arena::new());
    let context = Context { arena: arena_cell };
    let root =
        main_parser::parse(source, &context).map_err(|e| anyhow!("failed to parse: {}", e))?;

    Ok((context.arena.take(), root))
}
