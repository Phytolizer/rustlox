use crate::{object::LoxObject, token::Token};

pub trait Visitor<T> {
    fn visit_assign_expr(&mut self, expr: &Assign) -> T;
    fn visit_binary_expr(&mut self, expr: &Binary) -> T;
    fn visit_call_expr(&mut self, expr: &Call) -> T;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> T;
    fn visit_literal_expr(&mut self, expr: &Literal) -> T;
    fn visit_logical_expr(&mut self, expr: &Logical) -> T;
    fn visit_unary_expr(&mut self, expr: &Unary) -> T;
    fn visit_variable_expr(&mut self, expr: &Variable) -> T;
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Unary(Unary),
    Variable(Variable),
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Expr::Assign(a) => visitor.visit_assign_expr(a),
            Expr::Binary(b) => visitor.visit_binary_expr(b),
            Expr::Call(c) => visitor.visit_call_expr(c),
            Expr::Grouping(g) => visitor.visit_grouping_expr(g),
            Expr::Literal(l) => visitor.visit_literal_expr(l),
            Expr::Logical(l) => visitor.visit_logical_expr(l),
            Expr::Unary(u) => visitor.visit_unary_expr(u),
            Expr::Variable(v) => visitor.visit_variable_expr(v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LoxObject,
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}
