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
    
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
    
    pub fn print_stmt(&mut self, stmt: &Stmt) -> String {
        self.visit_stmt(stmt)
    }
    
    pub fn print_program(&mut self, program: &[Stmt]) -> String {
        let mut result = String::new();
        for stmt in program {
            result.push_str(&self.visit_stmt(stmt));
        }
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
            Expr::Unary { operator, right } => {
                let mut result = format!("{}Unary\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!("{}operator: {}\n", printer.indent(), operator.token_type)
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
            Expr::Call { callee, paren, arguments } => {
                let mut result = format!("{}Call\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}callee:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(callee))
                    )
                }));
                result.push_str(&self.nested(|printer| {
                    let mut args_result = format!("{}arguments:\n", printer.indent());
                    for (i, arg) in arguments.iter().enumerate() {
                        args_result.push_str(&printer.nested(|p| {
                            format!("{}[{}]:\n{}", p.indent(), i, p.nested(|p2| p2.visit_expr(arg)))
                        }));
                    }
                    args_result
                }));
                result
            }
            Expr::Comma { left, right } => {
                let mut result = format!("{}Comma\n", self.indent());
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
            Expr::Get { object, name } => {
                let mut result = format!("{}Get\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}object:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(object))
                    )
                }));
                result.push_str(&self.nested(|printer| {
                    format!("{}name: {}\n", printer.indent(), name.token_type)
                }));
                result
            }
            Expr::Set { object, name, value } => {
                let mut result = format!("{}Set\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}object:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(object))
                    )
                }));
                result.push_str(&self.nested(|printer| {
                    format!("{}name: {}\n", printer.indent(), name.token_type)
                }));
                result.push_str(&self.nested(|printer| {
                    format!(
                        "{}value:\n{}",
                        printer.indent(),
                        printer.nested(|p| p.visit_expr(value))
                    )
                }));
                result
            }
            Expr::Super { keyword, method } => {
                let mut result = format!("{}Super\n", self.indent());
                result.push_str(&self.nested(|printer| {
                    format!("{}keyword: {}\n", printer.indent(), keyword.token_type)
                }));
                result.push_str(&self.nested(|printer| {
                    format!("{}method: {}\n", printer.indent(), method.token_type)
                }));
                result
            }
            Expr::This { keyword } => {
                format!("{}This: {}\n", self.indent(), keyword.token_type)
            }
            Expr::Logical { left, operator, right } => {
                let mut result = format!("{}Logical\n", self.indent());
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
