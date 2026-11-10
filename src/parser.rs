use crate::ast::*;
use crate::lexer::{Spanned, Token};

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match self.current().token {
            Token::Fn => Ok(Item::Function(self.parse_function()?)),
            Token::Class => Ok(Item::Struct(self.parse_struct()?)),
            Token::Methods => Ok(Item::Impl(self.parse_impl()?)),
            Token::Enum => Ok(Item::Enum(self.parse_enum()?)),
            Token::Trait => Ok(Item::Trait(self.parse_trait()?)),
            Token::Use => Ok(Item::Use(self.parse_use()?)),
            Token::Const => Ok(Item::Const(self.parse_const()?)),
            Token::TypeKw => Ok(Item::TypeAlias(self.parse_type_alias()?)),
            _ => Err(self.error("expected 'fn', 'class', 'methods', 'enum', 'trait', 'use', 'const', or 'type'")),
        }
    }

    // --- Functions ---

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Function {
            name,
            generic_params,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(params);
        }

        loop {
            self.skip_newlines();
            if self.check(&Token::Self_) {
                self.advance();
                params.push(Param {
                    name: "self".into(),
                    ty: Type::Inferred,
                    is_self: true,
                    is_mut_self: false,
                });
            } else if self.check(&Token::Mut) && self.peek_is(&Token::Self_) {
                self.advance();
                self.advance();
                params.push(Param {
                    name: "self".into(),
                    ty: Type::Inferred,
                    is_self: true,
                    is_mut_self: true,
                });
            } else {
                let name = self.expect_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                params.push(Param {
                    name,
                    ty,
                    is_self: false,
                    is_mut_self: false,
                });
            }

            self.skip_newlines();
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }

        Ok(params)
    }

    // --- Generics ---

    fn try_parse_generic_params(&mut self) -> Result<Vec<GenericParam>, String> {
        if !self.check(&Token::Lt) {
            return Ok(Vec::new());
        }
        self.advance();
        let mut params = Vec::new();

        loop {
            self.skip_newlines();
            let name = self.expect_ident()?;
            let mut bounds = Vec::new();

            if self.check(&Token::Colon) {
                self.advance();
                loop {
                    bounds.push(self.expect_ident()?);
                    if !self.check(&Token::Plus) {
                        break;
                    }
                    self.advance();
                }
            }

            params.push(GenericParam { name, bounds });
            self.skip_newlines();
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(Token::Gt)?;
        Ok(params)
    }

    fn try_parse_type_args(&mut self) -> Result<Vec<Type>, String> {
        if !self.check(&Token::Lt) {
            return Ok(Vec::new());
        }
        self.advance();
        let mut args = Vec::new();

        loop {
            self.skip_newlines();
            args.push(self.parse_type()?);
            self.skip_newlines();
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(Token::Gt)?;
        Ok(args)
    }

    // --- Types ---

    fn parse_type(&mut self) -> Result<Type, String> {
        match &self.current().token {
            Token::IntType => {
                self.advance();
                Ok(Type::Int)
            }
            Token::FloatType => {
                self.advance();
                Ok(Type::Float)
            }
            Token::StringType => {
                self.advance();
                Ok(Type::String)
            }
            Token::BoolType => {
                self.advance();
                Ok(Type::Bool)
            }
            Token::CharType => {
                self.advance();
                Ok(Type::Char)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                if self.check(&Token::Lt) {
                    let args = self.try_parse_type_args()?;
                    Ok(Type::Parameterized(name, args))
                } else {
                    Ok(Type::Named(name))
                }
            }
            Token::LParen => {
                self.advance();
                self.expect(Token::RParen)?;
                Ok(Type::Void)
            }
            _ => Err(self.error(&format!("expected type, got {}", self.current().token))),
        }
    }

    // --- Structs ---

    fn parse_struct(&mut self) -> Result<Struct, String> {
        self.expect(Token::Class)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.skip_newlines();
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) {
            let fname = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            fields.push(Field { name: fname, ty });
            self.skip_newlines();
            if self.check(&Token::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(Struct {
            name,
            generic_params,
            fields,
        })
    }

    // --- Impl ---

    fn parse_impl(&mut self) -> Result<ImplBlock, String> {
        self.expect(Token::Methods)?;
        let generic_params = self.try_parse_generic_params()?;

        let first_name = self.expect_ident()?;
        let first_type_args = self.try_parse_type_args()?;

        // Check for `impl Trait for Type`
        let (trait_name, type_name, type_args) = if self.check(&Token::For) {
            self.advance();
            let type_name = self.expect_ident()?;
            let type_args = self.try_parse_type_args()?;
            (Some(first_name), type_name, type_args)
        } else {
            (None, first_name, first_type_args)
        };

        self.skip_newlines();
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) {
            methods.push(self.parse_function()?);
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(ImplBlock {
            generic_params,
            trait_name,
            type_name,
            type_args,
            methods,
        })
    }

    // --- Enum ---

    fn parse_enum(&mut self) -> Result<Enum, String> {
        self.expect(Token::Enum)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.skip_newlines();
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) {
            let vname = self.expect_ident()?;
            let mut fields = Vec::new();
            if self.check(&Token::LParen) {
                self.advance();
                if !self.check(&Token::RParen) {
                    loop {
                        fields.push(self.parse_type()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
            }
            variants.push(EnumVariant {
                name: vname,
                fields,
            });
            self.skip_newlines();
            if self.check(&Token::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(Enum {
            name,
            generic_params,
            variants,
        })
    }

    // --- Traits ---

    fn parse_trait(&mut self) -> Result<Trait, String> {
        self.expect(Token::Trait)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.skip_newlines();
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) {
            methods.push(self.parse_trait_method()?);
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(Trait {
            name,
            generic_params,
            methods,
        })
    }

    fn parse_trait_method(&mut self) -> Result<TraitMethod, String> {
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        self.skip_newlines();

        let body = if self.check(&Token::LBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(TraitMethod {
            name,
            generic_params,
            params,
            return_type,
            body,
        })
    }

    // --- Use ---

    fn parse_use(&mut self) -> Result<UseDecl, String> {
        self.expect(Token::Use)?;
        let mut path = Vec::new();
        path.push(self.expect_ident()?);

        let mut glob = false;
        let mut group = None;
        let mut alias = None;

        while self.check(&Token::Dot) || self.check(&Token::ColonColon) {
            self.advance();
            if self.check(&Token::Star) {
                self.advance();
                glob = true;
                break;
            }
            if self.check(&Token::LBrace) {
                self.advance();
                let mut items = Vec::new();
                loop {
                    self.skip_newlines();
                    items.push(self.expect_ident()?);
                    self.skip_newlines();
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
                self.expect(Token::RBrace)?;
                group = Some(items);
                break;
            }
            path.push(self.expect_ident()?);
        }

        if self.check(&Token::As) {
            self.advance();
            alias = Some(self.expect_ident()?);
        }

        self.skip_newlines();

        Ok(UseDecl {
            path,
            glob,
            group,
            alias,
        })
    }

    // --- Const ---

    fn parse_const(&mut self) -> Result<Const, String> {
        self.expect(Token::Const)?;
        let name = self.expect_ident()?;

        let ty = if self.check(&Token::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Inferred
        };

        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.skip_newlines();

        Ok(Const { name, ty, value })
    }

    // --- Type alias ---

    fn parse_type_alias(&mut self) -> Result<TypeAlias, String> {
        self.expect(Token::TypeKw)?;
        let name = self.expect_ident()?;
        let generic_params = self.try_parse_generic_params()?;
        self.expect(Token::Eq)?;
        let target = self.parse_type()?;
        self.skip_newlines();

        Ok(TypeAlias {
            name,
            generic_params,
            target,
        })
    }

    // --- Block ---

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(stmts)
    }

    // --- Statements ---

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.current().token {
            Token::Var => self.parse_var(),
            Token::Mut => self.parse_mut(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Match => self.parse_match(),
            Token::Break => {
                self.advance();
                self.skip_newlines();
                Ok(Statement::Break)
            }
            Token::Continue => {
                self.advance();
                self.skip_newlines();
                Ok(Statement::Continue)
            }
            _ => {
                let expr = self.parse_expr()?;

                if self.check(&Token::Eq) {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.skip_newlines();
                    return Ok(Statement::Assign {
                        target: expr,
                        value,
                    });
                }

                if let Some(op) = self.check_compound_assign() {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.skip_newlines();
                    return Ok(Statement::CompoundAssign {
                        target: expr,
                        op,
                        value,
                    });
                }

                self.skip_newlines();
                Ok(Statement::Expr(expr))
            }
        }
    }

    fn parse_var(&mut self) -> Result<Statement, String> {
        self.expect(Token::Var)?;
        let name = self.expect_ident()?;
        let ty = if self.check(&Token::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Inferred
        };
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.skip_newlines();
        Ok(Statement::Var { name, ty, value })
    }

    fn parse_mut(&mut self) -> Result<Statement, String> {
        self.expect(Token::Mut)?;
        let name = self.expect_ident()?;
        let ty = if self.check(&Token::Colon) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Inferred
        };
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.skip_newlines();
        Ok(Statement::Mut { name, ty, value })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.expect(Token::Return)?;
        if self.check(&Token::Newline)
            || self.check(&Token::RBrace)
            || self.check(&Token::Eof)
        {
            self.skip_newlines();
            return Ok(Statement::Return(None));
        }
        let expr = self.parse_expr()?;
        self.skip_newlines();
        Ok(Statement::Return(Some(expr)))
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        self.skip_newlines();
        let body = self.parse_block()?;

        let mut else_if_branches = Vec::new();
        let mut else_body = None;

        self.skip_newlines();
        while self.check(&Token::Else) {
            self.advance();
            if self.check(&Token::If) {
                self.advance();
                let cond = self.parse_expr()?;
                self.skip_newlines();
                let b = self.parse_block()?;
                else_if_branches.push((cond, b));
                self.skip_newlines();
            } else {
                self.skip_newlines();
                else_body = Some(self.parse_block()?);
                break;
            }
        }

        Ok(Statement::If {
            condition,
            body,
            else_if_branches,
            else_body,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.expect(Token::While)?;
        let condition = self.parse_expr()?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Statement, String> {
        self.expect(Token::For)?;
        let var = self.expect_ident()?;
        self.expect(Token::In)?;
        let iter = self.parse_expr()?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Ok(Statement::For { var, iter, body })
    }

    fn parse_match(&mut self) -> Result<Statement, String> {
        self.expect(Token::Match)?;
        let value = self.parse_expr()?;
        self.skip_newlines();
        self.expect(Token::LBrace)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;
            self.skip_newlines();

            let body = if self.check(&Token::LBrace) {
                self.parse_block()?
            } else {
                let stmt = self.parse_statement()?;
                vec![stmt]
            };

            arms.push(MatchArm { pattern, body });
            self.skip_newlines();
        }
        self.expect(Token::RBrace)?;

        Ok(Statement::Match { value, arms })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match &self.current().token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Int(_) | Token::Float(_) | Token::StringLit(_) | Token::True
            | Token::False | Token::CharLit(_) | Token::Minus => {
                let expr = self.parse_primary()?;
                Ok(Pattern::Literal(expr))
            }
            Token::Ident(name) => {
                let mut name = name.clone();
                self.advance();
                // Support qualified names like Color::Red
                while self.check(&Token::ColonColon) {
                    self.advance();
                    let segment = self.expect_ident()?;
                    name = format!("{name}::{segment}");
                }
                if self.check(&Token::LParen) {
                    self.advance();
                    let mut fields = Vec::new();
                    if !self.check(&Token::RParen) {
                        loop {
                            fields.push(self.expect_ident()?);
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Pattern::EnumVariant { name, fields })
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            _ => Err(self.error(&format!(
                "expected pattern, got {}",
                self.current().token
            ))),
        }
    }

    // --- Expression parsing (precedence climbing) ---

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match &self.current().token {
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_range()?;
        loop {
            let op = match &self.current().token {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::Lte,
                Token::GtEq => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_range()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr, String> {
        let left = self.parse_addition()?;
        if self.check(&Token::DotDot) {
            self.advance();
            let end = self.parse_addition()?;
            return Ok(Expr::Range {
                start: Some(Box::new(left)),
                end: Box::new(end),
                inclusive: false,
            });
        }
        if self.check(&Token::DotDotEq) {
            self.advance();
            let end = self.parse_addition()?;
            return Ok(Expr::Range {
                start: Some(Box::new(left)),
                end: Box::new(end),
                inclusive: true,
            });
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match &self.current().token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_power()?;
        loop {
            let op = match &self.current().token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let left = self.parse_unary()?;
        if self.check(&Token::DoubleStar) {
            self.advance();
            let right = self.parse_power()?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Power,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match &self.current().token {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&Token::LParen) {
                self.advance();
                let args = self.parse_args()?;
                self.expect(Token::RParen)?;
                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                };
            } else if self.check(&Token::Dot) {
                self.advance();
                let field = self.expect_ident()?;
                if self.check(&Token::LParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                    };
                } else {
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
            } else if self.check(&Token::LBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.check(&Token::As) {
                self.advance();
                let ty = self.parse_type()?;
                expr = Expr::Cast {
                    expr: Box::new(expr),
                    ty,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.current().clone();
        match &tok.token {
            Token::Int(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLit(n))
            }
            Token::Float(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::FloatLit(n))
            }
            Token::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLit(s))
            }
            Token::CharLit(c) => {
                let c = *c;
                self.advance();
                Ok(Expr::CharLit(c))
            }
            Token::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            Token::Self_ => {
                self.advance();
                Ok(Expr::Ident("self".into()))
            }
            Token::Ident(name) => {
                let mut name = name.clone();
                self.advance();

                // Support qualified names: Color::Red, Module::func
                while self.check(&Token::ColonColon) {
                    self.advance();
                    let segment = self.expect_ident()?;
                    name = format!("{name}::{segment}");
                }

                // Struct init: Name { field: val, ... }
                if self.check(&Token::LBrace)
                    && name.chars().next().is_some_and(|c| c.is_uppercase())
                    && !name.contains("::")
                {
                    self.advance();
                    self.skip_newlines();
                    let mut fields = Vec::new();
                    while !self.check(&Token::RBrace) {
                        let fname = self.expect_ident()?;
                        if self.check(&Token::Colon) {
                            self.advance();
                            let val = self.parse_expr()?;
                            fields.push((fname, val));
                        } else {
                            fields.push((fname.clone(), Expr::Ident(fname)));
                        }
                        self.skip_newlines();
                        if self.check(&Token::Comma) {
                            self.advance();
                        }
                        self.skip_newlines();
                    }
                    self.expect(Token::RBrace)?;
                    return Ok(Expr::StructInit { name, fields });
                }

                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                if !self.check(&Token::RBracket) {
                    loop {
                        self.skip_newlines();
                        elems.push(self.parse_expr()?);
                        self.skip_newlines();
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.skip_newlines();
                self.expect(Token::RBracket)?;
                Ok(Expr::ArrayLit(elems))
            }
            Token::Pipe => {
                self.advance();
                let mut params = Vec::new();
                if !self.check(&Token::Pipe) {
                    loop {
                        params.push(self.expect_ident()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(Token::Pipe)?;
                let body = self.parse_expr()?;
                Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            _ => Err(self.error(&format!(
                "unexpected token: {}",
                tok.token
            ))),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(args);
        }
        loop {
            self.skip_newlines();
            args.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        Ok(args)
    }

    // --- Helpers ---

    fn current(&self) -> &Spanned {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(&self.current().token) == std::mem::discriminant(expected)
    }

    fn peek_is(&self, expected: &Token) -> bool {
        self.tokens.get(self.pos + 1).is_some_and(|t| {
            std::mem::discriminant(&t.token) == std::mem::discriminant(expected)
        })
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!(
                "expected {expected}, got {}",
                self.current().token
            )))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match &self.current().token {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error(&format!(
                "expected identifier, got {}",
                self.current().token
            ))),
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.check(&Token::Eof)
    }

    fn check_compound_assign(&self) -> Option<BinOp> {
        match &self.current().token {
            Token::PlusEq => Some(BinOp::Add),
            Token::MinusEq => Some(BinOp::Sub),
            Token::StarEq => Some(BinOp::Mul),
            Token::SlashEq => Some(BinOp::Div),
            _ => None,
        }
    }

    fn error(&self, msg: &str) -> String {
        let span = self.current();
        format!("line {}:{}: {msg}", span.line, span.col)
    }
}
