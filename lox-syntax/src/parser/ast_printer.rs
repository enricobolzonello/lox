use crate::tokenizer::token::Literal;

use super::ast::{Expr, ExprVisitor, Stmt, StmtVisitor};

pub struct TreePrinter {
    indent_level: usize,
}

impl TreePrinter {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn nested<F>(&mut self, f: F) -> String
    where
        F: FnOnce(&mut Self) -> String,
    {
        self.indent_level += 1;
        let result = f(self);
        self.indent_level -= 1;
        result
    }
}

impl ExprVisitor<String> for TreePrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let mut result = format!("{}Binary\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!("{}operator: {}\n", printer.indent(), operator.token_type)
                }));
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}left:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(left))
                    )
                }));
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}right:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(right))
                    )
                }));
                result
            }
            Expr::Grouping { expression } => {
                let mut result = format!("{}Grouping\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}expression:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(expression))
                    )
                }));
                result
            }
            Expr::Literal { value } => {
                format!(
                    "{}Literal: {}\n",
                    self.indent(),
                    match value {
                        Literal::Number(n)=>n.to_string(),
                        Literal::String(s)=>format!("\"{}\"",s),
                        Literal::Bool(b) => format!("\"{}\"",b),
                        Literal::Null => format!("\"nil\""),
                    }
                )
            }
            Expr::Variable { name } => {
                format!(
                    "{}Variable: {}\n",
                    self.indent(),
                    name.literal.clone().unwrap_or(Literal::String("None".to_string()))
                )
            }
            Expr::Assign { name, value } => {
                let mut result = format!("{}Assign\n", self.indent());
                result.push_str(
                    &self.nested(|printer| {
                        format!("{}name: {}\n", printer.indent(), name.token_type)
                    }),
                );
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}value:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(value))
                    )
                }));
                result
            }
            // Add other cases as needed
            _ => format!("{}[Other Expression Type]\n", self.indent()),
        }
    }
}

impl StmtVisitor<String> for TreePrinter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Expression { expression } => {
                let mut result = format!("{}ExpressionStmt\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}expression:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(expression))
                    )
                }));
                result
            }
            Stmt::Print { expression } => {
                let mut result = format!("{}PrintStmt\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}expression:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(expression))
                    )
                }));
                result
            }
            Stmt::Var { name, initializer } => {
                let mut result = format!("{}VarStmt\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}name: {}\n",
                        printer.indent(),
                        name.literal
                            .clone()
                            .unwrap_or(Literal::String("None".to_string()))
                    )
                }));

                if let Some(init) = initializer {
                    result.push_str(&self.nested(|printer| {
                        format!(
                            "{}initializer:\n{}",
                            printer.indent(),
                            printer.nested(|p| p.visit_expr(init))
                        )
                    }));
                } else {
                    result.push_str(
                        &self.nested(|printer| format!("{}initializer: nil\n", printer.indent())),
                    );
                }
                result
            }
            Stmt::Block { statements } => {
                let mut result = format!("{}BlockStmt\n", self.indent());
                result
                    .push_str(&self.nested(|printer| format!("{}statements:\n", printer.indent())));

                for stmt in statements {
                    result.push_str(&self.nested(|printer| printer.nested(|p| p.visit_stmt(stmt))));
                }
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut result = format!("{}IfStmt\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}condition:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(condition))
                    )
                }));
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}then_branch:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_stmt(then_branch))
                    )
                }));

                if let Some(else_stmt) = else_branch {
                    result.push_str(&self.nested(|printer| {
                        format!(
                            "{}else_branch:\n{}",
                            printer.indent(),
                            printer.nested(|p| p.visit_stmt(else_stmt))
                        )
                    }));
                }
                result
            }
            // Add cases for other statement types
            _ => format!("{}[Other Statement Type]\n", self.indent()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::ast::{Expr, Stmt, StmtVisitor}, tokenizer::{token::Literal, Token, TokenType}};

    use super::TreePrinter;

    #[test]
    fn test_pretty_print() {
        // Create a simple program: var x = 1 + 2; print x;
        let program = vec![
            Stmt::Var {
                name: Token {
                    token_type: TokenType::VAR,
                    literal: Some(Literal::String("x".to_string())),
                    line: 0,
                },
                initializer: Some(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(1.0),
                    }),
                    operator: Token {
                        token_type: TokenType::PLUS,
                        literal: None,
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(2.0),
                    }),
                }),
            },
            Stmt::Print {
                expression: Expr::Variable {
                    name: Token {
                        token_type: TokenType::VAR,
                        literal: Some(Literal::String("x".to_string())),
                        line: 0,
                    },
                },
            },
        ];

        let mut tree_printer = TreePrinter::new();
        for stmt in &program {
            print!("{}", tree_printer.visit_stmt(stmt));
        }
    }
}
