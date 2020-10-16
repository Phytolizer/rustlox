use std::sync::Arc;

use crate::{object::Object, token::Token};

pub trait Visitor<T> {
    fn visit_assign_expr(&mut self, expr: &Assign) -> T;
    fn visit_binary_expr(&mut self, expr: &Binary) -> T;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> T;
    fn visit_literal_expr(&mut self, expr: &Literal) -> T;
    fn visit_unary_expr(&mut self, expr: &Unary) -> T;
    fn visit_variable_expr(&mut self, expr: &Variable) -> T;
}

pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Expr::Assign(a) => visitor.visit_assign_expr(a),
            Expr::Binary(b) => visitor.visit_binary_expr(b),
            Expr::Grouping(g) => visitor.visit_grouping_expr(g),
            Expr::Literal(l) => visitor.visit_literal_expr(l),
            Expr::Unary(u) => visitor.visit_unary_expr(u),
            Expr::Variable(v) => visitor.visit_variable_expr(v),
        }
    }
}

pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

pub struct Literal {
    pub value: Arc<Object>,
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct Variable {
    pub name: Token,
}
