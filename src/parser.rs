use crate::{expr::Binary, expr::Expr, expr::Grouping, expr::Literal, expr::Unary, object::Object, stmt::Stmt, token::{Token, TokenKind}};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.at_end() {
            statements.push(self.statement());
        }

        statements
    }

    fn statement(&mut self) -> Stmt {
        if self.matches(vec![TokenKind::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        Stmt::Print(value)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        Stmt::Expression(expr)
    }

    fn expression(&mut self) -> Result<Expr, (Token, String)> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, (Token, String)> {
        let mut expr = self.comparison()?;

        while self.matches(vec![TokenKind::BangEqual, TokenKind::EqualEqual]) {
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

        while self.matches(vec![
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

        while self.matches(vec![TokenKind::Minus, TokenKind::Plus]) {
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

        while self.matches(vec![TokenKind::Slash, TokenKind::Star]) {
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
        if self.matches(vec![TokenKind::Bang, TokenKind::Minus]) {
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
        if self.matches(vec![TokenKind::False]) {
            return Ok(Expr::Literal(Literal {
                value: Object::new_bool(false),
            }));
        }
        if self.matches(vec![TokenKind::True]) {
            return Ok(Expr::Literal(Literal {
                value: Object::new_bool(true),
            }));
        }
        if self.matches(vec![TokenKind::Nil]) {
            return Ok(Expr::Literal(Literal {
                value: Object::nil(),
            }));
        }

        if self.matches(vec![TokenKind::Number, TokenKind::String]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous().literal.clone(),
            }));
        }

        if self.matches(vec![TokenKind::LParen]) {
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

    fn matches(&mut self, kinds: Vec<TokenKind>) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }

        false
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
