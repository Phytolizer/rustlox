use std::{
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{
    environment::Environment,
    expr::{self, Expr},
    object::LoxObject,
    object::Object,
    runtime_error::RuntimeError,
    stmt,
    token::Token,
    token::TokenKind,
};

fn check_number_operand(operator: &Token, operand: LoxObject) -> Result<(), RuntimeError> {
    if operand.read().unwrap().is_number() {
        Ok(())
    } else {
        Err(RuntimeError::new(
            operator.clone(),
            String::from("Operand must be a number."),
        ))
    }
}

fn check_number_operands(
    left: LoxObject,
    operator: &Token,
    right: LoxObject,
) -> Result<(), RuntimeError> {
    if left.read().unwrap().is_number() && right.read().unwrap().is_number() {
        Ok(())
    } else {
        Err(RuntimeError::new(
            operator.clone(),
            String::from("Operands must be numbers."),
        ))
    }
}

pub struct Interpreter {
    pub globals: Arc<RwLock<Environment>>,
    environment: Arc<RwLock<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Arc::new(RwLock::new(Environment::new()));

        globals.write().unwrap().define(
            "clock",
            Object::new_builtin_function(0, |_args| {
                Object::new_number(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                )
            }),
        );

        Self {
            globals: globals.clone(),
            environment: globals,
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

    pub fn execute_block(
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

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxObject, RuntimeError> {
        expr.accept(self)
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<(), RuntimeError> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Result<(), RuntimeError> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value.read().unwrap());
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
        if self.evaluate(&stmt.condition)?.read().unwrap().as_bool() {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &stmt::While) -> Result<(), RuntimeError> {
        while self.evaluate(&stmt.condition)?.read().unwrap().as_bool() {
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &stmt::Function) -> Result<(), RuntimeError> {
        let function = Object::new_function(stmt.clone());
        self.environment
            .write()
            .unwrap()
            .define(&stmt.name.lexeme, function);
        Ok(())
    }
}

impl expr::Visitor<Result<LoxObject, RuntimeError>> for Interpreter {
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<LoxObject, RuntimeError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        Ok(match expr.operator.kind {
            TokenKind::Minus => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(
                    left.read().unwrap().as_number() - right.read().unwrap().as_number(),
                )
            }
            TokenKind::Slash => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(
                    left.read().unwrap().as_number() / right.read().unwrap().as_number(),
                )
            }
            TokenKind::Star => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_number(
                    left.read().unwrap().as_number() * right.read().unwrap().as_number(),
                )
            }
            TokenKind::Plus => {
                if left.read().unwrap().is_number() && right.read().unwrap().is_number() {
                    Object::new_number(
                        left.read().unwrap().as_number() + right.read().unwrap().as_number(),
                    )
                } else if left.read().unwrap().is_string() && right.read().unwrap().is_string() {
                    Object::new_string(
                        left.read().unwrap().to_string()
                            + right.read().unwrap().as_string().as_ref(),
                    )
                } else {
                    return Err(RuntimeError::new(
                        expr.operator.clone(),
                        String::from("Operands must be two numbers or two strings."),
                    ));
                }
            }
            TokenKind::Greater => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(
                    left.read().unwrap().as_number() > right.read().unwrap().as_number(),
                )
            }
            TokenKind::GreaterEqual => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(
                    left.read().unwrap().as_number() >= right.read().unwrap().as_number(),
                )
            }
            TokenKind::Less => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(
                    left.read().unwrap().as_number() < right.read().unwrap().as_number(),
                )
            }
            TokenKind::LessEqual => {
                check_number_operands(left.clone(), &expr.operator, right.clone())?;
                Object::new_bool(
                    left.read().unwrap().as_number() <= right.read().unwrap().as_number(),
                )
            }
            TokenKind::EqualEqual => {
                Object::new_bool(left.read().unwrap().eq(&right.read().unwrap()))
            }
            TokenKind::BangEqual => {
                Object::new_bool(left.read().unwrap().eq(&right.read().unwrap()))
            }
            _ => unreachable!(),
        })
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Result<LoxObject, RuntimeError> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &expr::Literal) -> Result<LoxObject, RuntimeError> {
        Ok(expr.value.clone())
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Result<LoxObject, RuntimeError> {
        let right = self.evaluate(&expr.right)?;

        Ok(match expr.operator.kind {
            TokenKind::Bang => Object::new_bool(!right.read().unwrap().as_bool()),
            TokenKind::Minus => {
                check_number_operand(&expr.operator, right.clone())?;
                Object::new_number(-right.read().unwrap().as_number())
            }
            _ => unreachable!(),
        })
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Result<LoxObject, RuntimeError> {
        self.environment.read().unwrap().get(&expr.name)
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Result<LoxObject, RuntimeError> {
        let value = self.evaluate(&expr.value)?;

        self.environment
            .write()
            .unwrap()
            .assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Result<LoxObject, RuntimeError> {
        let left = self.evaluate(&expr.left)?;

        match expr.operator.kind {
            TokenKind::Or => {
                if left.read().unwrap().as_bool() {
                    return Ok(left);
                }
            }
            TokenKind::And => {
                if !left.read().unwrap().as_bool() {
                    return Ok(left);
                }
            }
            _ => unreachable!(),
        }

        self.evaluate(&expr.right)
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) -> Result<LoxObject, RuntimeError> {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments = vec![];
        for arg in &expr.arguments {
            arguments.push(self.evaluate(arg)?);
        }

        if !callee.read().unwrap().is_callable() {
            return Err(RuntimeError::new(
                expr.paren.clone(),
                String::from("Can only call functions and classes."),
            ));
        }

        if arguments.len() != callee.read().unwrap().arity() {
            return Err(RuntimeError::new(
                expr.paren.clone(),
                format!(
                    "Expected {} arguments but got {}.",
                    callee.read().unwrap().arity(),
                    arguments.len()
                ),
            ));
        }

        let ret = callee.write().unwrap().call(self, arguments)?;
        Ok(ret)
    }
}
