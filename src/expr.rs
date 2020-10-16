use std::sync::Arc;

use crate::{object::Object, token::Token};

pub trait Visitor<T> {
    fn visit_binary_expr(&mut self, binary: &Binary) -> T;
    fn visit_grouping_expr(&mut self, grouping: &Grouping) -> T;
    fn visit_literal_expr(&mut self, literal: &Literal) -> T;
    fn visit_unary_expr(&mut self, unary: &Unary) -> T;
}

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Expr::Binary(b) => visitor.visit_binary_expr(b),
            Expr::Grouping(g) => visitor.visit_grouping_expr(g),
            Expr::Literal(l) => visitor.visit_literal_expr(l),
            Expr::Unary(u) => visitor.visit_unary_expr(u),
        }
    }
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
