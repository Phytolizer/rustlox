use crate::{
    expr::Assign,
    expr::Binary,
    expr::Expr,
    expr::Grouping,
    expr::Literal,
    expr::Logical,
    expr::Unary,
    expr::Variable,
    object::Object,
    stmt::Block,
    stmt::Expression,
    stmt::If,
    stmt::Print,
    stmt::Stmt,
    stmt::Var,
    stmt::While,
    token::{Token, TokenKind},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, (Token, String)> {
        let mut statements = vec![];

        while !self.at_end() {
            if let Some(decl) = self.declaration() {
                statements.push(decl);
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let value = if self.matches(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match value {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, (Token, String)> {
        let name = self
            .consume(TokenKind::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer = None;
        if self.matches(&[TokenKind::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(Var { name, initializer }))
    }

    fn statement(&mut self) -> Result<Stmt, (Token, String)> {
        if self.matches(&[TokenKind::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.matches(&[TokenKind::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.matches(&[TokenKind::LBrace]) {
            return Ok(Stmt::Block(Block {
                statements: self.block()?,
            }));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenKind::LParen, "Expect '(' after 'for'.")?;
        let initializer = if self.matches(&[TokenKind::Semicolon]) {
            None
        } else if self.matches(&[TokenKind::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after for condition.")?;
        let increment = if self.check(TokenKind::RParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenKind::RParen, "Expect ')' after for clauses.")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(Block {
                statements: vec![
                    body,
                    Stmt::Expression(Expression {
                        expression: increment,
                    }),
                ],
            });
        }

        let condition = condition.unwrap_or_else(|| {
            Expr::Literal(Literal {
                value: Object::new_bool(true),
            })
        });

        body = Stmt::While(While {
            condition,
            body: Box::new(body),
        });

        if let Some(initializer) = initializer {
            body = Stmt::Block(Block {
                statements: vec![initializer, body],
            });
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenKind::LParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(&[TokenKind::Else]) {
            Some(self.statement()?)
        } else {
            None
        }
        .map(|eb| Box::new(eb));

        Ok(Stmt::If(If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, (Token, String)> {
        let mut statements = vec![];

        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(decl) = self.declaration() {
                statements.push(decl);
            }
        }

        self.consume(TokenKind::RBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, (Token, String)> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Print { expression: value }))
    }

    fn while_statement(&mut self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenKind::LParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RParen, "Expect ')' after while condition")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While(While { condition, body }))
    }

    fn expression_statement(&mut self) -> Result<Stmt, (Token, String)> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Expression { expression: expr }))
    }

    fn expression(&mut self) -> Result<Expr, (Token, String)> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, (Token, String)> {
        let expr = self.or()?;

        if self.matches(&[TokenKind::Equal]) {
            let equals = self.previous().clone();
            let value = Box::new(self.assignment()?);

            if let Expr::Variable(v) = &expr {
                let name = v.name.clone();
                return Ok(Expr::Assign(Assign { name, value }));
            }

            Self::error(&equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.and()?;

        while self.matches(&[TokenKind::Or]) {
            let operator = self.previous().clone();
            let right = Box::new(self.and()?);
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenKind::And]) {
            let operator = self.previous().clone();
            let right = Box::new(self.equality()?);
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, (Token, String)> {
        if self.matches(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, (Token, String)> {
        if self.matches(&[TokenKind::False]) {
            return Ok(Expr::Literal(Literal {
                value: Object::new_bool(false),
            }));
        }
        if self.matches(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal {
                value: Object::new_bool(true),
            }));
        }
        if self.matches(&[TokenKind::Nil]) {
            return Ok(Expr::Literal(Literal {
                value: Object::nil(),
            }));
        }

        if self.matches(&[TokenKind::Number, TokenKind::String]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous().literal.clone(),
            }));
        }

        if self.matches(&[TokenKind::Identifier]) {
            return Ok(Expr::Variable(Variable {
                name: self.previous().clone(),
            }));
        }

        if self.matches(&[TokenKind::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Grouping {
                expression: Box::new(expr),
            }));
        }

        Err(Self::error(self.peek(), "Expect expression."))
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, (Token, String)> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        Err(Self::error(self.peek(), message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn error(token: &Token, message: &str) -> (Token, String) {
        crate::error_at_token(token, message);
        (token.clone(), message.to_string())
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        if kinds.iter().cloned().any(|k| self.check(k)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
