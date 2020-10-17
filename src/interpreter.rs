use std::sync::{Arc, RwLock};

use crate::{
    environment::Environment,
    expr::{self, Expr},
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

pub struct Interpreter {
    environment: Arc<RwLock<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Arc::new(RwLock::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[stmt::Stmt]) {
        if let Some(e) = statements.iter().find_map(|s| self.execute(s).err()) {
            crate::runtime_error(e);
        }
    }

    fn execute(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &[stmt::Stmt],
        environment: Environment,
    ) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();

        self.environment = Arc::new(RwLock::new(environment));

        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.environment = previous;
                return Err(e);
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Arc<Object>, RuntimeError> {
        expr.accept(self)
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<(), RuntimeError> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Result<(), RuntimeError> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) -> Result<(), RuntimeError> {
        let value = if let Some(initializer) = &stmt.initializer {
            Some(self.evaluate(initializer)?)
        } else {
            None
        };
        self.environment
            .write()
            .unwrap()
            .define(&stmt.name.lexeme, value.unwrap_or_else(Object::nil));
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) -> Result<(), RuntimeError> {
        self.execute_block(
            &stmt.statements,
            Environment::new_enclosed(self.environment.clone()),
        )
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) -> Result<(), RuntimeError> {
        if self.evaluate(&stmt.condition)?.as_bool() {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }
}

impl expr::Visitor<Result<Arc<Object>, RuntimeError>> for Interpreter {
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<Arc<Object>, RuntimeError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        Ok(match expr.operator.kind {
            TokenKind::Minus => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(left.as_number() - right.as_number())
            }
            TokenKind::Slash => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(left.as_number() / right.as_number())
            }
            TokenKind::Star => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(left.as_number() * right.as_number())
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    Object::new_number(left.as_number() + right.as_number())
                } else if left.is_string() && right.is_string() {
                    Object::new_string(left.to_string() + right.as_string().as_ref())
                } else {
                    return Err(RuntimeError::new(
                        expr.operator.clone(),
                        String::from("Operands must be two numbers or two strings."),
                    ));
                }
            }
            TokenKind::Greater => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(left.as_number() > right.as_number())
            }
            TokenKind::GreaterEqual => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(left.as_number() >= right.as_number())
            }
            TokenKind::Less => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(left.as_number() < right.as_number())
            }
            TokenKind::LessEqual => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(left.as_number() <= right.as_number())
            }
            TokenKind::EqualEqual => Object::new_bool(left == right),
            TokenKind::BangEqual => Object::new_bool(left != right),
            _ => unreachable!(),
        })
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Result<Arc<Object>, RuntimeError> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &expr::Literal) -> Result<Arc<Object>, RuntimeError> {
        Ok(expr.value.clone())
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Result<Arc<Object>, RuntimeError> {
        let right = self.evaluate(&expr.right)?;

        Ok(match expr.operator.kind {
            TokenKind::Bang => Object::new_bool(!right.as_bool()),
            TokenKind::Minus => {
                check_number_operand(&expr.operator, right.clone())?;
                Object::new_number(-right.as_number())
            }
            _ => unreachable!(),
        })
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Result<Arc<Object>, RuntimeError> {
        self.environment.read().unwrap().get(&expr.name)
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Result<Arc<Object>, RuntimeError> {
        let value = self.evaluate(&expr.value)?;

        self.environment
            .write()
            .unwrap()
            .assign(&expr.name, value.clone())?;
        Ok(value)
    }
}
