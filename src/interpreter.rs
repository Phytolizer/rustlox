use std::sync::Arc;

use crate::{
    expr::Expr,
    expr::{self},
    object::Object,
    runtime_error::RuntimeError,
    stmt,
    token::Token,
    token::TokenKind,
};

fn check_number_operand(operator: &Token, operand: Arc<Object>) -> Result<(), RuntimeError> {
    if operand.is_number() {
        Ok(())
    } else {
        Err(RuntimeError::new(
            operator.clone(),
            String::from("Operand must be a number."),
        ))
    }
}

fn check_number_operands(
    left: Arc<Object>,
    operator: &Token,
    right: Arc<Object>,
) -> Result<(), RuntimeError> {
    if left.is_number() && right.is_number() {
        Ok(())
    } else {
        Err(RuntimeError::new(
            operator.clone(),
            String::from("Operands must be numbers."),
        ))
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, statements: &[stmt::Stmt]) {
        statements
            .iter()
            .find_map(|s| self.execute(s).err())
            .map(|e| crate::runtime_error(e));
    }

    fn execute(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Arc<Object>, RuntimeError> {
        expr.accept(self)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) {
        self.evaluate(&stmt.expression);
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) {
        let value = self.evaluate(&stmt.expression);
        println!("{}", value);
    }
}

impl expr::Visitor<Result<Arc<Object>, RuntimeError>> for Interpreter {
    fn visit_binary_expr(
        &mut self,
        binary: &crate::expr::Binary,
    ) -> Result<Arc<Object>, RuntimeError> {
        let left = self.evaluate(&binary.left)?;
        let right = self.evaluate(&binary.right)?;

        Ok(match binary.operator.kind {
            TokenKind::Minus => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_number(left.as_number() - right.as_number())
            }
            TokenKind::Slash => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_number(left.as_number() / right.as_number())
            }
            TokenKind::Star => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_number(left.as_number() * right.as_number())
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    Object::new_number(left.as_number() + right.as_number())
                } else if left.is_string() && right.is_string() {
                    Object::new_string(left.to_string() + right.as_string().as_ref())
                } else {
                    return Err(RuntimeError::new(
                        binary.operator.clone(),
                        String::from("Operands must be two numbers or two strings."),
                    ));
                }
            }
            TokenKind::Greater => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_bool(left.as_number() > right.as_number())
            }
            TokenKind::GreaterEqual => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_bool(left.as_number() >= right.as_number())
            }
            TokenKind::Less => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_bool(left.as_number() < right.as_number())
            }
            TokenKind::LessEqual => {
                check_number_operands(left.clone(), &binary.operator, right.clone())?;
                Object::new_bool(left.as_number() <= right.as_number())
            }
            TokenKind::EqualEqual => Object::new_bool(left == right),
            TokenKind::BangEqual => Object::new_bool(left != right),
            _ => unreachable!(),
        })
    }

    fn visit_grouping_expr(
        &mut self,
        grouping: &crate::expr::Grouping,
    ) -> Result<Arc<Object>, RuntimeError> {
        self.evaluate(&grouping.expression)
    }

    fn visit_literal_expr(
        &mut self,
        literal: &crate::expr::Literal,
    ) -> Result<Arc<Object>, RuntimeError> {
        Ok(literal.value.clone())
    }

    fn visit_unary_expr(
        &mut self,
        unary: &crate::expr::Unary,
    ) -> Result<Arc<Object>, RuntimeError> {
        let right = self.evaluate(&unary.right)?;

        Ok(match unary.operator.kind {
            TokenKind::Bang => Object::new_bool(!right.as_bool()),
            TokenKind::Minus => {
                check_number_operand(&unary.operator, right.clone())?;
                Object::new_number(-right.as_number())
            }
            _ => unreachable!(),
        })
    }
}
