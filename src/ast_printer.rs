use crate::expr::{self, Expr};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
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

impl expr::Visitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, binary: &crate::expr::Binary) -> String {
        self.parenthesize(&binary.operator.lexeme, &[&binary.left, &binary.right])
    }

    fn visit_grouping_expr(&mut self, grouping: &crate::expr::Grouping) -> String {
        self.parenthesize("group", &[&grouping.expression])
    }

    fn visit_literal_expr(&mut self, literal: &crate::expr::Literal) -> String {
        literal.value.to_string()
    }

    fn visit_unary_expr(&mut self, unary: &crate::expr::Unary) -> String {
        self.parenthesize(&unary.operator.lexeme, &[&unary.right])
    }

    fn visit_variable_expr(&mut self, variable: &expr::Variable) -> String {
        variable.name.lexeme.clone()
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> String {
        self.parenthesize(&(format!("assign {}", expr.name.lexeme)), &[&expr.value])
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
    }
}
