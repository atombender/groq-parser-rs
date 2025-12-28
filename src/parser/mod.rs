pub mod operators;
#[cfg(test)]
mod tests;

use crate::ast::*;
use crate::parser::operators::*;
use crate::tokenizer::Tokenizer;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub pos: Position,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.message, self.pos)
    }
}

impl std::error::Error for ParseError {}

/// Configuration for the parser.
#[derive(Debug, Clone, Default)]
pub struct ParserConfig {
    /// Set of parameter names that are valid (without the $ prefix).
    /// If validate_params is true and a param is referenced that's not in this set,
    /// parsing will fail.
    pub params: HashSet<String>,

    /// Whether to validate that referenced parameters exist in the params set.
    /// Default: true (matches Go behavior).
    pub validate_params: bool,
}

impl ParserConfig {
    /// Create a new config with parameter validation enabled and an empty param set.
    /// This matches Go's default behavior where params must be provided.
    pub fn new() -> Self {
        ParserConfig {
            params: HashSet::new(),
            validate_params: true,
        }
    }

    /// Create a config that skips parameter validation.
    pub fn without_param_validation() -> Self {
        ParserConfig {
            params: HashSet::new(),
            validate_params: false,
        }
    }

    /// Add a parameter name to the valid set.
    pub fn with_param(mut self, name: &str) -> Self {
        self.params.insert(name.to_string());
        self
    }

    /// Add multiple parameter names to the valid set.
    pub fn with_params<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for name in names {
            self.params.insert(name.as_ref().to_string());
        }
        self
    }
}

pub struct Parser<'a> {
    tk: Tokenizer<'a>,
    buf_tok: Token,
    buf_lit: &'a str,
    buf_pos: usize,
    has_buf: bool,
    config: ParserConfig,
    /// Tracks all parameter references found during parsing (name, position)
    referenced_params: Vec<(String, Position)>,
}

impl<'a> Parser<'a> {
    /// Create a new parser without parameter validation.
    /// Use `new_with_config` for parameter validation (Go's default behavior).
    pub fn new(src: &'a str) -> Self {
        Self::new_with_config(src, ParserConfig::without_param_validation())
    }

    /// Create a new parser with the given configuration.
    pub fn new_with_config(src: &'a str, config: ParserConfig) -> Self {
        Parser {
            tk: Tokenizer::new(src),
            buf_tok: Token::Illegal,
            buf_lit: "",
            buf_pos: 0,
            has_buf: false,
            config,
            referenced_params: Vec::new(),
        }
    }

    #[inline]
    fn scan(&mut self) -> (Token, &'a str, usize) {
        if self.has_buf {
            self.has_buf = false;
            return (self.buf_tok, self.buf_lit, self.buf_pos);
        }

        let (tok, lit, pos) = self.tk.scan();
        self.buf_tok = tok;
        self.buf_lit = lit;
        self.buf_pos = pos;
        (tok, lit, pos)
    }

    #[inline]
    fn scan_ignore_whitespace(&mut self) -> (Token, &'a str, usize) {
        loop {
            let (tok, lit, pos) = self.scan();
            if tok != Token::Whitespace {
                return (tok, lit, pos);
            }
        }
    }

    #[inline]
    fn unscan(&mut self) {
        self.has_buf = true;
    }

    #[inline]
    fn make_pos(&self, start: usize, end: usize) -> Position {
        Position { start, end }
    }

    #[inline]
    fn make_token_pos(&self, start: usize, literal: &str) -> Position {
        Position {
            start,
            end: start + literal.len(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let (tok, _, _) = self.scan_ignore_whitespace();
        if tok == Token::EOF {
            return Err(Box::new(ParseError {
                message: "no query".to_string(),
                pos: self.make_pos(0, 0),
            }));
        }
        self.unscan();

        self.parse_function_definitions()?;

        let result = self.parse_general_expression(1, false, false)?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::EOF {
            return Err(Box::new(ParseError {
                message: "unable to parse entire expression".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        // Validate parameter references if enabled
        if self.config.validate_params {
            for (param_name, param_pos) in &self.referenced_params {
                if !self.config.params.contains(param_name) {
                    return Err(Box::new(ParseError {
                        message: format!(
                            "param ${} referenced, but not provided",
                            param_name
                        ),
                        pos: *param_pos,
                    }));
                }
            }
        }

        Ok(result)
    }

    fn parse_function_definitions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (tok, lit, _) = self.scan_ignore_whitespace();
            if tok == Token::Name && lit == "fn" {
                let _func_def = self.parse_function_definition()?;
            } else {
                self.unscan();
                break;
            }
        }
        Ok(())
    }

    fn parse_function_definition(
        &mut self,
    ) -> Result<FunctionDefinition, Box<dyn std::error::Error>> {
        let (namespace, name) = self.parse_function_name()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::ParenLeft {
            return Err(Box::new(ParseError {
                message: "expected '(' following function name".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        let params = self.parse_function_parameters()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::ParenRight {
            return Err(Box::new(ParseError {
                message: "expected ')' following function arguments".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::EqualSign {
            return Err(Box::new(ParseError {
                message: "expected '=' following ()".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        let body = self.parse_function_body()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::Semicolon {
            return Err(Box::new(ParseError {
                message: "expected ';' at the end of function definition".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        Ok(FunctionDefinition {
            pos: self.make_pos(pos, pos),
            id: FunctionID { namespace, name },
            body,
            parameters: params,
        })
    }

    fn parse_function_name(&mut self) -> Result<(String, String), Box<dyn std::error::Error>> {
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name {
            return Err(Box::new(ParseError {
                message: "expected function namespace".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }
        let namespace = lit.to_string();

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::DoubleColon {
            return Err(Box::new(ParseError {
                message: "expected '::' followed by a function name".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name {
            return Err(Box::new(ParseError {
                message: "expected a function name".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }
        let name = lit.to_string();

        Ok((namespace, name))
    }

    fn parse_function_parameters(
        &mut self,
    ) -> Result<Vec<FunctionParamDefinition>, Box<dyn std::error::Error>> {
        let mut params = Vec::new();
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name || !lit.starts_with('$') {
            return Err(Box::new(ParseError {
                message: "expected parameter name".to_string(),
                pos: self.make_pos(pos, pos),
            }));
        }

        params.push(FunctionParamDefinition {
            index: 0,
            name: lit[1..].to_string(),
        });

        Ok(params)
    }

    fn parse_function_body(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        self.parse_general_expression(1, false, false)
    }

    fn parse_list(&mut self) -> Result<Vec<Expr>, Box<dyn std::error::Error>> {
        let mut exprs = Vec::new();
        loop {
            match self.parse_general_expression(1, false, true) {
                Ok(expr) => exprs.push(expr),
                Err(e) => {
                    if e.to_string() == "EmptyExpression" {
                        break;
                    }
                    return Err(e);
                }
            }

            let (tok, _, _) = self.scan_ignore_whitespace();
            if tok != Token::Comma {
                self.unscan();
                break;
            }
        }
        Ok(exprs)
    }

    fn parse_general_expression(
        &mut self,
        min_precedence: i32,
        immediate_lhs: bool,
        may_be_empty: bool,
    ) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr: Expr;

        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if is_prefix_operator(tok) {
            let (precedence, associativity) = precedence_and_associativity(tok);
            let rhs_result = if associativity == Associativity::Left {
                self.parse_general_expression(precedence + 1, false, true)
            } else {
                self.parse_general_expression(precedence, false, true)
            };

            if tok == Token::DotDotDot {
                match rhs_result {
                    Ok(rhs) => {
                        expr = Expr::Prefix(PrefixOperator {
                            pos: self.make_token_pos(pos, lit),
                            operator: tok,
                            rhs: Box::new(rhs),
                        });
                    }
                    Err(e) if e.to_string() == "EmptyExpression" => {
                        expr = Expr::Ellipsis(Ellipsis {
                            pos: self.make_token_pos(pos, lit),
                        });
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let rhs = rhs_result?;
                expr = Expr::Prefix(PrefixOperator {
                    pos: self.make_token_pos(pos, lit),
                    operator: tok,
                    rhs: Box::new(rhs),
                });
            }
        } else {
            self.unscan();
            if let Some(e) = self.parse_atom(immediate_lhs)? {
                expr = e;
            } else {
                if may_be_empty {
                    return Err("EmptyExpression".into());
                }
                let (a_tok, a_lit, a_pos) = self.scan();
                let seen = if a_tok == Token::EOF {
                    "end-of-file".to_string()
                } else {
                    format!("token {:?}", a_lit)
                };
                return Err(Box::new(ParseError {
                    message: format!("unexpected {}, expected expression", seen),
                    pos: self.make_token_pos(a_pos, a_lit),
                }));
            }
        }

        loop {
            let (tok, ident, pos) = self.scan_ignore_whitespace();

            let operator;

            if !is_operator(tok) {
                self.unscan();
                if tok == Token::BraceLeft {
                    operator = Token::Pipe;
                } else if tok == Token::BracketLeft {
                    if HIGHEST_PRECEDENCE < min_precedence {
                        break;
                    }

                    let rhs = self.parse_chained_bracketed_expression()?;
                    match rhs {
                        Some(Expr::Attribute(attr)) => {
                            expr = Expr::Dot(DotOperator {
                                pos: self.make_token_pos(pos, ident),
                                lhs: Box::new(expr),
                                rhs: Box::new(Expr::Attribute(attr)),
                            });
                            continue;
                        }
                        Some(Expr::Subscript(sub)) => {
                            if let Expr::Range(range) = *sub.value {
                                let p = sub.pos;
                                expr = Expr::Slice(Slice {
                                    pos: p,
                                    lhs: Box::new(expr),
                                    range: Subscript {
                                        pos: p,
                                        value: Box::new(Expr::Range(range)),
                                    },
                                });
                            } else {
                                expr = Expr::Element(Element {
                                    pos: sub.pos,
                                    lhs: Box::new(expr),
                                    idx: sub,
                                });
                            }
                            continue;
                        }
                        Some(Expr::Constraint(cons)) => {
                            expr = Expr::Filter(Filter {
                                pos: cons.pos,
                                lhs: Box::new(expr),
                                constraint: cons,
                            });
                            continue;
                        }
                        Some(Expr::ArrayTraversal(at)) => {
                            expr = Expr::ArrayTraversal(ArrayTraversal {
                                pos: at.pos,
                                expr: Box::new(expr),
                            });
                            continue;
                        }
                        _ => {
                            self.unscan();
                            break;
                        }
                    }
                } else if let Expr::Postfix(ref p) = expr {
                    if p.operator == Token::Arrow && tok == Token::Name {
                        operator = Token::Dot;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                operator = tok;
            }

            let (precedence, associativity) = precedence_and_associativity(operator);
            if precedence < min_precedence {
                self.unscan();
                break;
            }

            if is_postfix_operator(tok) {
                expr = Expr::Postfix(PostfixOperator {
                    pos: self.make_token_pos(pos, ident),
                    lhs: Box::new(expr),
                    operator: tok,
                });
                continue;
            }

            let rhs_has_immediate_lhs = operator == Token::Pipe || operator == Token::Dot;

            let rhs = if associativity == Associativity::Left {
                self.parse_general_expression(precedence + 1, rhs_has_immediate_lhs, false)?
            } else {
                self.parse_general_expression(precedence, rhs_has_immediate_lhs, false)?
            };

            match operator {
                Token::Dot => {
                    expr = Expr::Dot(DotOperator {
                        pos: self.make_token_pos(pos, ident),
                        lhs: Box::new(expr),
                        rhs: Box::new(rhs),
                    });
                }
                Token::Pipe => match rhs {
                    Expr::Object(obj) => {
                        expr = Expr::Projection(Projection {
                            pos: self.make_token_pos(pos, ident),
                            lhs: Box::new(expr),
                            object: obj,
                        });
                    }
                    Expr::FunctionCall(func) => {
                        expr = Expr::FunctionPipe(FunctionPipe {
                            pos: self.make_token_pos(pos, ident),
                            lhs: Box::new(expr),
                            func,
                        });
                    }
                    _ => return Err("object or function expected after pipe".into()),
                },
                _ => {
                    let bin_op = BinaryOperator {
                        pos: self.make_token_pos(pos, ident),
                        operator,
                        lhs: Box::new(expr),
                        rhs: Box::new(rhs),
                    };

                    if operator == Token::DotDot || operator == Token::DotDotDot {
                        expr = Expr::Range(Range {
                            pos: bin_op.pos,
                            start: bin_op.lhs,
                            end: bin_op.rhs,
                            inclusive: operator == Token::DotDot,
                        });
                    } else {
                        expr = Expr::Binary(bin_op);
                    }
                }
            }
        }

        Ok(expr)
    }

    fn parse_chained_bracketed_expression(
        &mut self,
    ) -> Result<Option<Expr>, Box<dyn std::error::Error>> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BracketLeft {
            return Ok(None);
        }

        let expr_result = self.parse_general_expression(1, false, true);
        let expr = match expr_result {
            Ok(e) => Some(e),
            Err(e) => {
                if e.to_string() == "EmptyExpression" {
                    None
                } else {
                    return Err(e);
                }
            }
        };

        let (tok_end, lit_end, pos_end) = self.scan_ignore_whitespace();
        let range_end = pos_end + lit_end.len();

        if tok_end != Token::BracketRight {
            return Err(Box::new(ParseError {
                message: "expected ']' following expression".to_string(),
                pos: self.make_pos(pos_start, range_end),
            }));
        }

        if expr.is_none() {
            return Ok(Some(Expr::ArrayTraversal(ArrayTraversal {
                pos: self.make_pos(pos_start, range_end),
                expr: Box::new(Expr::Everything(Everything {
                    pos: self.make_pos(0, 0),
                })),
            })));
        }
        let expr = expr.unwrap();

        if let Expr::Literal(Literal::String(s)) = &expr {
            return Ok(Some(Expr::Attribute(Attribute {
                pos: self.make_pos(pos_start, range_end),
                name: s.value.clone(),
            })));
        }

        if let Expr::Range(_) = &expr {
            if !expr.is_subscript_expression() {
                return Err(Box::new(ParseError {
                    message: "subscript ranges must have integer endpoints".to_string(),
                    pos: self.make_pos(pos_start, range_end),
                }));
            }
            return Ok(Some(Expr::Subscript(Subscript {
                pos: self.make_pos(pos_start, range_end),
                value: Box::new(expr),
            })));
        }

        if expr.is_subscript_expression() {
            return Ok(Some(Expr::Subscript(Subscript {
                pos: self.make_pos(pos_start, range_end),
                value: Box::new(expr),
            })));
        }

        Ok(Some(Expr::Constraint(Constraint {
            pos: self.make_pos(pos_start, range_end),
            expression: Box::new(expr),
        })))
    }

    fn parse_atom(
        &mut self,
        immediate_lhs: bool,
    ) -> Result<Option<Expr>, Box<dyn std::error::Error>> {
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        match tok {
            Token::Name => {
                if let Some(stripped) = lit.strip_prefix('$') {
                    let param_pos = self.make_token_pos(pos, lit);
                    // Track this parameter reference for validation
                    self.referenced_params
                        .push((stripped.to_string(), param_pos));
                    return Ok(Some(Expr::Param(Param {
                        pos: param_pos,
                        name: stripped.to_string(),
                    })));
                }

                if let Some(func) = self.parse_function_expression(tok, lit, pos)? {
                    return Ok(Some(Expr::FunctionCall(func)));
                }

                Ok(Some(Expr::Attribute(Attribute {
                    pos: self.make_token_pos(pos, lit),
                    name: lit.to_string(),
                })))
            }
            Token::Asterisk => Ok(Some(Expr::Everything(Everything {
                pos: self.make_token_pos(pos, lit),
            }))),
            Token::At => Ok(Some(Expr::This(This {
                pos: self.make_token_pos(pos, lit),
            }))),
            Token::Hat => Ok(Some(Expr::Parent(Parent {
                pos: self.make_token_pos(pos, lit),
            }))),
            Token::AscOperator | Token::DescOperator | Token::InOperator | Token::MatchOperator => {
                Ok(Some(Expr::Attribute(Attribute {
                    pos: self.make_token_pos(pos, lit),
                    name: lit.to_string(),
                })))
            }
            Token::Integer => {
                // Try parsing as i64 first, fall back to f64 for large numbers
                if let Ok(value) = lit.parse::<i64>() {
                    Ok(Some(Expr::Literal(Literal::Integer(IntegerLiteral {
                        pos: self.make_token_pos(pos, lit),
                        value,
                    }))))
                } else {
                    // Integer too large for i64, parse as float
                    Ok(Some(Expr::Literal(Literal::Float(FloatLiteral {
                        pos: self.make_token_pos(pos, lit),
                        value: lit.parse()?,
                    }))))
                }
            }
            Token::Float => Ok(Some(Expr::Literal(Literal::Float(FloatLiteral {
                pos: self.make_token_pos(pos, lit),
                value: lit.parse()?,
            })))),
            Token::String => {
                // Parse the string value, handling escape sequences
                let inner = &lit[1..lit.len() - 1];
                let value = unescape_string(inner)?;
                Ok(Some(Expr::Literal(Literal::String(StringLiteral {
                    pos: self.make_token_pos(pos, lit),
                    value,
                }))))
            }
            Token::Bool => Ok(Some(Expr::Literal(Literal::Boolean(BooleanLiteral {
                pos: self.make_token_pos(pos, lit),
                value: lit == "true",
            })))),
            Token::Null => Ok(Some(Expr::Literal(Literal::Null(NullLiteral {
                pos: self.make_token_pos(pos, lit),
            })))),
            Token::ParenLeft => {
                let expr = self.parse_parenthesis_expr(pos, lit)?;
                Ok(Some(expr))
            }
            Token::BracketLeft => {
                self.unscan();
                if immediate_lhs {
                    let expr = self.parse_chained_bracketed_expression()?;
                    Ok(expr)
                } else {
                    let expr = self.parse_array_expression()?;
                    Ok(Some(expr))
                }
            }
            Token::BraceLeft => {
                self.unscan();
                let expr = self.parse_object_expression()?;
                Ok(Some(expr))
            }
            _ => {
                self.unscan();
                Ok(None)
            }
        }
    }

    fn parse_parenthesis_expr(
        &mut self,
        pos: usize,
        lit: &str,
    ) -> Result<Expr, Box<dyn std::error::Error>> {
        let expr = self.parse_general_expression(1, false, false)?;
        let (tok, _, tok_pos) = self.scan_ignore_whitespace();

        if tok == Token::Comma {
            let mut tuple_members = vec![expr];
            let mut current_tok = tok;

            while current_tok == Token::Comma {
                let next_expr = self.parse_general_expression(1, false, false)?;
                tuple_members.push(next_expr);
                let (t, _, _) = self.scan_ignore_whitespace();
                current_tok = t;
            }

            if current_tok != Token::ParenRight {
                return Err(Box::new(ParseError {
                    message: "expected ')' following parenthesized expression".to_string(),
                    pos: self.make_pos(pos, tok_pos),
                }));
            }

            return Ok(Expr::Tuple(Tuple {
                pos: self.make_token_pos(pos, lit),
                members: tuple_members,
            }));
        }

        if tok != Token::ParenRight {
            return Err(Box::new(ParseError {
                message: "expected ')' following parenthesized expression".to_string(),
                pos: self.make_pos(pos, tok_pos),
            }));
        }

        Ok(Expr::Group(Group {
            pos: self.make_token_pos(pos, lit),
            expression: Box::new(expr),
        }))
    }

    fn parse_array_expression(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BracketLeft {
            return Err("Expected '['".into());
        }

        let (peek_tok, _, _) = self.scan_ignore_whitespace();
        let mut exprs = Vec::new();

        if peek_tok != Token::BracketRight {
            self.unscan();
            exprs = self.parse_list()?;
        } else {
            self.unscan();
        }

        let (tok_end, lit_end, pos_end) = self.scan_ignore_whitespace();
        let range_end = pos_end + lit_end.len();

        if tok_end != Token::BracketRight {
            return Err(Box::new(ParseError {
                message: "expected ']' following array body".to_string(),
                pos: self.make_pos(pos_start, range_end),
            }));
        }

        Ok(Expr::Array(Array {
            pos: self.make_pos(pos_start, range_end),
            expressions: exprs,
        }))
    }

    fn parse_object_expression(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BraceLeft {
            return Err("Expected '{'".into());
        }

        let mut exprs = Vec::new();
        let (peek_tok, _, _) = self.scan_ignore_whitespace();

        if peek_tok != Token::BraceRight {
            self.unscan();
            exprs = self.parse_list()?;
        } else {
            self.unscan();
        }

        let (tok_end, lit_end, pos_end) = self.scan_ignore_whitespace();
        let range_end = pos_end + lit_end.len();

        if tok_end != Token::BraceRight {
            return Err(Box::new(ParseError {
                message: "expected '}' following object body".to_string(),
                pos: self.make_pos(pos_start, range_end),
            }));
        }

        Ok(Expr::Object(Object {
            pos: self.make_pos(pos_start, range_end),
            expressions: exprs,
        }))
    }

    fn parse_function_expression(
        &mut self,
        name_token: Token,
        name: &'a str,
        name_pos: usize,
    ) -> Result<Option<FunctionCall>, Box<dyn std::error::Error>> {
        let (tok, lit, pos) = self.scan();

        match tok {
            Token::DoubleColon => {
                if name_token != Token::Name {
                    return Err(Box::new(ParseError {
                        message: "expected valid namespace identifier before '::'".to_string(),
                        pos: self.make_token_pos(pos, lit),
                    }));
                }

                let (f_tok, f_lit, f_pos) = self.scan_ignore_whitespace();
                if f_tok != Token::Name {
                    return Err(Box::new(ParseError {
                        message: "expected a function following namespace expression".to_string(),
                        pos: self.make_token_pos(f_pos, f_lit),
                    }));
                }

                let func_opt = self.parse_function_expression(Token::DoubleColon, f_lit, f_pos)?;
                if let Some(mut func) = func_opt {
                    func.namespace = name.to_string();
                    func.pos.start = name_pos;
                    return Ok(Some(func));
                } else {
                    return Err(Box::new(ParseError {
                        message: "expected a function following namespace expression".to_string(),
                        pos: self.make_token_pos(f_pos, f_lit),
                    }));
                }
            }
            Token::ParenLeft => {
                // Fall through to parse arguments
            }
            Token::Whitespace => {
                let (next_tok, _, _) = self.scan();
                if next_tok == Token::BracketLeft
                    || next_tok == Token::BraceLeft
                    || is_infix_operator(next_tok)
                    || is_postfix_operator(next_tok)
                {
                    self.unscan();
                    self.unscan();
                    return Ok(None);
                }
                if next_tok != Token::ParenLeft {
                    self.unscan();
                    self.unscan();
                    return Ok(None);
                }
                // Fall through to parse arguments
            }
            _ => {
                self.unscan();
                return Ok(None);
            }
        }

        // Parse function arguments
        let mut exprs = Vec::new();
        let (peek_tok, _, _) = self.scan_ignore_whitespace();
        if peek_tok != Token::ParenRight {
            self.unscan();
            exprs = self.parse_list()?;
        } else {
            self.unscan();
        }

        let (tok_end, lit_end, pos_end) = self.scan_ignore_whitespace();
        let range_end = pos_end + lit_end.len();
        if tok_end != Token::ParenRight {
            return Err(Box::new(ParseError {
                message: "expected ')' following function arguments".to_string(),
                pos: self.make_pos(pos, range_end),
            }));
        }

        Ok(Some(FunctionCall {
            namespace: String::new(),
            pos: self.make_token_pos(name_pos, name),
            name: name.to_string(),
            arguments: exprs,
        }))
    }
}

fn unescape_string(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000c}'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    if chars.peek() == Some(&'{') {
                        chars.next(); // consume '{'
                        let mut hex = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                hex.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        if chars.peek() == Some(&'}') {
                            chars.next();
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16)
                            && let Some(c) = char::from_u32(code)
                        {
                            result.push(c);
                        }
                    } else {
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            if let Some(c) = chars.next() {
                                hex.push(c);
                            }
                        }
                        if let Ok(u) = u16::from_str_radix(&hex, 16) {
                            // Check for surrogate pair
                            if (0xD800..=0xDBFF).contains(&u) {
                                // High surrogate - look for low surrogate
                                if chars.next() == Some('\\') && chars.next() == Some('u') {
                                    let mut hex2 = String::with_capacity(4);
                                    for _ in 0..4 {
                                        if let Some(c) = chars.next() {
                                            hex2.push(c);
                                        }
                                    }
                                    if let Ok(u2) = u16::from_str_radix(&hex2, 16)
                                        && (0xDC00..=0xDFFF).contains(&u2)
                                    {
                                        let code = 0x10000
                                            + (((u as u32) - 0xD800) << 10)
                                            + ((u2 as u32) - 0xDC00);
                                        if let Some(c) = char::from_u32(code) {
                                            result.push(c);
                                        }
                                    }
                                }
                            } else if let Some(c) = char::from_u32(u as u32) {
                                result.push(c);
                            }
                        }
                    }
                }
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}
