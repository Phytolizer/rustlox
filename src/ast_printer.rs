use crate::expr::{Expr, Visitor};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        format!(
            "({} {})",
            name,
            exprs
                .iter()
                .map(|e| e.accept(self))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, binary: &crate::expr::Binary) -> String {
        self.parenthesize(&binary.operator.lexeme, vec![&binary.left, &binary.right])
    }

    fn visit_grouping_expr(&mut self, grouping: &crate::expr::Grouping) -> String {
        self.parenthesize("group", vec![&grouping.expression])
    }

    fn visit_literal_expr(&mut self, literal: &crate::expr::Literal) -> String {
        literal.value.to_string()
    }

    fn visit_unary_expr(&mut self, unary: &crate::expr::Unary) -> String {
        self.parenthesize(&unary.operator.lexeme, vec![&unary.right])
    }
}
