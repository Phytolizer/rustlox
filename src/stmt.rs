use crate::expr::Expr;

pub trait Visitor<T> {
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> T;
    fn visit_print_stmt(&mut self, stmt: &Print) -> T;
}

pub enum Stmt {
    Expression(Expression),
    Print(Print),
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Stmt::Expression(e) => visitor.visit_expression_stmt(e),
            Stmt::Print(p) => visitor.visit_print_stmt(p),
        }
    }
}

pub struct Expression {
    pub expression: Expr,
}

pub struct Print {
    pub expression: Expr,
}
