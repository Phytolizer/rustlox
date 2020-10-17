use crate::{expr::Expr, token::Token};

pub trait Visitor<T> {
    fn visit_block_stmt(&mut self, stmt: &Block) -> T;
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> T;
    fn visit_print_stmt(&mut self, stmt: &Print) -> T;
    fn visit_var_stmt(&mut self, stmt: &Var) -> T;
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Stmt::Block(b) => visitor.visit_block_stmt(b),
            Stmt::Expression(e) => visitor.visit_expression_stmt(e),
            Stmt::Print(p) => visitor.visit_print_stmt(p),
            Stmt::Var(v) => visitor.visit_var_stmt(v),
        }
    }
}

pub struct Block {
    pub statements: Vec<Stmt>,
}

pub struct Expression {
    pub expression: Expr,
}

pub struct Print {
    pub expression: Expr,
}

pub struct Var {
    pub name: Token,
    pub initializer: Option<Expr>,
}
