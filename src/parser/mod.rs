pub mod operators;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fmt;

use crate::ast::*;
use crate::parser::operators::*;
use crate::tokenizer::Tokenizer;

/// An error encountered while parsing a GROQ query.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The query does not conform to GROQ syntax.
    Syntax { message: String, pos: Position },
    /// An expression exceeded [`ParserConfig::max_expression_depth`].
    ExpressionDepthExceeded {
        max_depth: usize,
        depth: usize,
        pos: Position,
    },
}

impl ParseError {
    fn syntax(message: impl Into<String>, pos: Position) -> Self {
        Self::Syntax {
            message: message.into(),
            pos,
        }
    }

    fn is_empty_expression(&self) -> bool {
        matches!(
            self,
            Self::Syntax { message, .. } if message == "EmptyExpression"
        )
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { message, pos } => write!(f, "{message} at {pos:?}"),
            Self::ExpressionDepthExceeded {
                max_depth,
                depth,
                pos,
            } => write!(
                f,
                "expression depth {depth} exceeds configured maximum of {max_depth} at {pos:?}"
            ),
        }
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

    /// Whether to collect comments and include them in the parse result.
    /// Default: false (comments are silently skipped).
    pub preserve_comments: bool,

    /// Maximum number of expression nodes on any root-to-leaf AST path.
    /// When unset, expression depth is unlimited.
    pub max_expression_depth: Option<usize>,
}

impl ParserConfig {
    /// Create a new config with parameter validation enabled and an empty param set.
    /// This matches Go's default behavior where params must be provided.
    pub fn new() -> Self {
        ParserConfig {
            params: HashSet::new(),
            validate_params: true,
            preserve_comments: false,
            max_expression_depth: None,
        }
    }

    /// Create a config that skips parameter validation.
    pub fn without_param_validation() -> Self {
        ParserConfig {
            params: HashSet::new(),
            validate_params: false,
            preserve_comments: false,
            max_expression_depth: None,
        }
    }

    /// Add a parameter name to the valid set.
    pub fn with_param(mut self, name: &str) -> Self {
        self.params.insert(name.to_string());
        self
    }

    /// Enable comment preservation in the parse result.
    pub fn with_comments(mut self) -> Self {
        self.preserve_comments = true;
        self
    }

    /// Limit the number of expression nodes on any root-to-leaf AST path.
    pub fn with_max_expression_depth(mut self, max_expression_depth: usize) -> Self {
        self.max_expression_depth = Some(max_expression_depth);
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

struct Parsed<T> {
    node: T,
    depth: usize,
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
    /// Comments collected during parsing (when preserve_comments is enabled).
    comments: Vec<Comment>,
    expression_parse_depth: usize,
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
            comments: Vec::new(),
            expression_parse_depth: 0,
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
            if tok == Token::Whitespace {
                continue;
            }
            if tok == Token::Comment {
                if self.config.preserve_comments {
                    self.comments.push(Comment {
                        pos: Position {
                            start: pos,
                            end: pos + lit.len(),
                        },
                        text: lit.to_string(),
                    });
                }
                continue;
            }
            return (tok, lit, pos);
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

    fn ensure_expression_depth(&self, depth: usize, pos: Position) -> Result<(), ParseError> {
        if let Some(max_expression_depth) = self.config.max_expression_depth
            && depth > max_expression_depth
        {
            return Err(ParseError::ExpressionDepthExceeded {
                max_depth: max_expression_depth,
                depth,
                pos,
            });
        }
        Ok(())
    }

    fn child_expression_depth<I>(&self, child_depths: I, pos: Position) -> Result<usize, ParseError>
    where
        I: IntoIterator<Item = usize>,
    {
        let depth = child_depths
            .into_iter()
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        self.ensure_expression_depth(depth, pos)?;
        Ok(depth)
    }

    pub fn parse(&mut self) -> Result<ParseResult, ParseError> {
        let (tok, _, _) = self.scan_ignore_whitespace();
        if tok == Token::EOF {
            return Err(ParseError::syntax("no query", self.make_pos(0, 0)));
        }
        self.unscan();

        let functions = self.parse_function_definitions()?;

        let expr = self.parse_general_expression(1, false, false)?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::EOF {
            return Err(ParseError::syntax(
                "unable to parse entire expression",
                self.make_pos(pos, pos),
            ));
        }

        // Validate parameter references if enabled
        if self.config.validate_params {
            for (param_name, param_pos) in &self.referenced_params {
                if !self.config.params.contains(param_name) {
                    return Err(ParseError::syntax(
                        format!("param ${param_name} referenced, but not provided"),
                        *param_pos,
                    ));
                }
            }
        }

        let comments = std::mem::take(&mut self.comments);
        Ok(ParseResult {
            expr: expr.node,
            functions,
            comments,
        })
    }

    fn parse_function_definitions(&mut self) -> Result<Vec<FunctionDefinition>, ParseError> {
        let mut functions = Vec::new();
        loop {
            let (tok, lit, _) = self.scan_ignore_whitespace();
            if tok == Token::Name && lit == "fn" {
                functions.push(self.parse_function_definition()?);
            } else {
                self.unscan();
                break;
            }
        }
        Ok(functions)
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDefinition, ParseError> {
        let (namespace, name) = self.parse_function_name()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::ParenLeft {
            return Err(ParseError::syntax(
                "expected '(' following function name",
                self.make_pos(pos, pos),
            ));
        }

        let params = self.parse_function_parameters()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::ParenRight {
            return Err(ParseError::syntax(
                "expected ')' following function arguments",
                self.make_pos(pos, pos),
            ));
        }

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::EqualSign {
            return Err(ParseError::syntax(
                "expected '=' following ()",
                self.make_pos(pos, pos),
            ));
        }

        let body = self.parse_function_body()?;

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::Semicolon {
            return Err(ParseError::syntax(
                "expected ';' at the end of function definition",
                self.make_pos(pos, pos),
            ));
        }

        Ok(FunctionDefinition {
            pos: self.make_pos(pos, pos),
            id: FunctionID { namespace, name },
            body,
            parameters: params,
        })
    }

    fn parse_function_name(&mut self) -> Result<(String, String), ParseError> {
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name {
            return Err(ParseError::syntax(
                "expected function namespace",
                self.make_pos(pos, pos),
            ));
        }
        let namespace = lit.to_string();

        let (tok, _, pos) = self.scan_ignore_whitespace();
        if tok != Token::DoubleColon {
            return Err(ParseError::syntax(
                "expected '::' followed by a function name",
                self.make_pos(pos, pos),
            ));
        }

        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name {
            return Err(ParseError::syntax(
                "expected a function name",
                self.make_pos(pos, pos),
            ));
        }
        let name = lit.to_string();

        Ok((namespace, name))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FunctionParamDefinition>, ParseError> {
        let mut params = Vec::new();
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        if tok != Token::Name || !lit.starts_with('$') {
            return Err(ParseError::syntax(
                "expected parameter name",
                self.make_pos(pos, pos),
            ));
        }

        params.push(FunctionParamDefinition {
            index: 0,
            name: lit[1..].to_string(),
        });

        Ok(params)
    }

    fn parse_function_body(&mut self) -> Result<Expr, ParseError> {
        Ok(self.parse_general_expression(1, false, false)?.node)
    }

    fn parse_list(&mut self) -> Result<Vec<Parsed<Expr>>, ParseError> {
        let mut exprs = Vec::new();
        loop {
            match self.parse_general_expression(1, false, true) {
                Ok(expr) => exprs.push(expr),
                Err(e) => {
                    if e.is_empty_expression() {
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
    ) -> Result<Parsed<Expr>, ParseError> {
        let expression_parse_depth = self.expression_parse_depth.saturating_add(1);
        self.ensure_expression_depth(
            expression_parse_depth,
            self.make_pos(self.buf_pos, self.buf_pos),
        )?;
        self.expression_parse_depth = expression_parse_depth;
        let result =
            self.parse_general_expression_inner(min_precedence, immediate_lhs, may_be_empty);
        self.expression_parse_depth -= 1;
        result
    }

    fn parse_general_expression_inner(
        &mut self,
        min_precedence: i32,
        immediate_lhs: bool,
        may_be_empty: bool,
    ) -> Result<Parsed<Expr>, ParseError> {
        let mut expr: Parsed<Expr>;

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
                        let operator_pos = self.make_token_pos(pos, lit);
                        let depth = self.child_expression_depth([rhs.depth], operator_pos)?;
                        expr = Parsed {
                            node: Expr::Prefix(PrefixOperator {
                                pos: operator_pos,
                                operator: tok,
                                rhs: Box::new(rhs.node),
                            }),
                            depth,
                        };
                    }
                    Err(e) if e.is_empty_expression() => {
                        expr = Parsed {
                            node: Expr::Ellipsis(Ellipsis {
                                pos: self.make_token_pos(pos, lit),
                            }),
                            depth: 1,
                        };
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let rhs = rhs_result?;
                let operator_pos = self.make_token_pos(pos, lit);
                let depth = self.child_expression_depth([rhs.depth], operator_pos)?;
                expr = Parsed {
                    node: Expr::Prefix(PrefixOperator {
                        pos: operator_pos,
                        operator: tok,
                        rhs: Box::new(rhs.node),
                    }),
                    depth,
                };
            }
        } else {
            self.unscan();
            if let Some(e) = self.parse_atom(immediate_lhs)? {
                expr = e;
            } else {
                if may_be_empty {
                    return Err(ParseError::syntax(
                        "EmptyExpression",
                        self.make_pos(self.buf_pos, self.buf_pos),
                    ));
                }
                let (a_tok, a_lit, a_pos) = self.scan();
                let seen = if a_tok == Token::EOF {
                    "end-of-file".to_string()
                } else {
                    format!("token {:?}", a_lit)
                };
                return Err(ParseError::syntax(
                    format!("unexpected {seen}, expected expression"),
                    self.make_token_pos(a_pos, a_lit),
                ));
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
                        Some(Parsed {
                            node: Expr::Attribute(attr),
                            depth: rhs_depth,
                        }) => {
                            let operator_pos = self.make_token_pos(pos, ident);
                            let depth =
                                self.child_expression_depth([expr.depth, rhs_depth], operator_pos)?;
                            expr = Parsed {
                                node: Expr::Dot(DotOperator {
                                    pos: operator_pos,
                                    lhs: Box::new(expr.node),
                                    rhs: Box::new(Expr::Attribute(attr)),
                                }),
                                depth,
                            };
                            continue;
                        }
                        Some(Parsed {
                            node: Expr::Subscript(sub),
                            depth: rhs_depth,
                        }) => {
                            let depth = expr.depth.saturating_add(1).max(rhs_depth);
                            self.ensure_expression_depth(depth, sub.pos)?;
                            if let Expr::Range(range) = *sub.value {
                                let p = sub.pos;
                                expr = Parsed {
                                    node: Expr::Slice(Slice {
                                        pos: p,
                                        lhs: Box::new(expr.node),
                                        range: Subscript {
                                            pos: p,
                                            value: Box::new(Expr::Range(range)),
                                        },
                                    }),
                                    depth,
                                };
                            } else {
                                expr = Parsed {
                                    node: Expr::Element(Element {
                                        pos: sub.pos,
                                        lhs: Box::new(expr.node),
                                        idx: sub,
                                    }),
                                    depth,
                                };
                            }
                            continue;
                        }
                        Some(Parsed {
                            node: Expr::Constraint(cons),
                            depth: rhs_depth,
                        }) => {
                            let depth = expr.depth.saturating_add(1).max(rhs_depth);
                            self.ensure_expression_depth(depth, cons.pos)?;
                            expr = Parsed {
                                node: Expr::Filter(Filter {
                                    pos: cons.pos,
                                    lhs: Box::new(expr.node),
                                    constraint: cons,
                                }),
                                depth,
                            };
                            continue;
                        }
                        Some(Parsed {
                            node: Expr::ArrayTraversal(at),
                            ..
                        }) => {
                            let depth = self.child_expression_depth([expr.depth], at.pos)?;
                            expr = Parsed {
                                node: Expr::ArrayTraversal(ArrayTraversal {
                                    pos: at.pos,
                                    expr: Box::new(expr.node),
                                }),
                                depth,
                            };
                            continue;
                        }
                        _ => {
                            self.unscan();
                            break;
                        }
                    }
                } else if let Expr::Postfix(ref p) = expr.node {
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
                let operator_pos = self.make_token_pos(pos, ident);
                let depth = self.child_expression_depth([expr.depth], operator_pos)?;
                expr = Parsed {
                    node: Expr::Postfix(PostfixOperator {
                        pos: operator_pos,
                        lhs: Box::new(expr.node),
                        operator: tok,
                    }),
                    depth,
                };
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
                    let operator_pos = self.make_token_pos(pos, ident);
                    let depth =
                        self.child_expression_depth([expr.depth, rhs.depth], operator_pos)?;
                    expr = Parsed {
                        node: Expr::Dot(DotOperator {
                            pos: operator_pos,
                            lhs: Box::new(expr.node),
                            rhs: Box::new(rhs.node),
                        }),
                        depth,
                    };
                }
                Token::Pipe => match rhs.node {
                    Expr::Object(obj) => {
                        let operator_pos = self.make_token_pos(pos, ident);
                        let depth = expr.depth.saturating_add(1).max(rhs.depth);
                        self.ensure_expression_depth(depth, operator_pos)?;
                        expr = Parsed {
                            node: Expr::Projection(Projection {
                                pos: operator_pos,
                                lhs: Box::new(expr.node),
                                object: obj,
                            }),
                            depth,
                        };
                    }
                    Expr::FunctionCall(func) => {
                        let operator_pos = self.make_token_pos(pos, ident);
                        let depth = expr.depth.saturating_add(1).max(rhs.depth);
                        self.ensure_expression_depth(depth, operator_pos)?;
                        expr = Parsed {
                            node: Expr::FunctionPipe(FunctionPipe {
                                pos: operator_pos,
                                lhs: Box::new(expr.node),
                                func,
                            }),
                            depth,
                        };
                    }
                    _ => {
                        return Err(ParseError::syntax(
                            "object or function expected after pipe",
                            self.make_token_pos(pos, ident),
                        ));
                    }
                },
                _ => {
                    let operator_pos = self.make_token_pos(pos, ident);
                    let depth =
                        self.child_expression_depth([expr.depth, rhs.depth], operator_pos)?;
                    let bin_op = BinaryOperator {
                        pos: operator_pos,
                        operator,
                        lhs: Box::new(expr.node),
                        rhs: Box::new(rhs.node),
                    };

                    if operator == Token::DotDot || operator == Token::DotDotDot {
                        expr = Parsed {
                            node: Expr::Range(Range {
                                pos: bin_op.pos,
                                start: bin_op.lhs,
                                end: bin_op.rhs,
                                inclusive: operator == Token::DotDot,
                            }),
                            depth,
                        };
                    } else {
                        expr = Parsed {
                            node: Expr::Binary(bin_op),
                            depth,
                        };
                    }
                }
            }
        }

        Ok(expr)
    }

    fn parse_chained_bracketed_expression(&mut self) -> Result<Option<Parsed<Expr>>, ParseError> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BracketLeft {
            return Ok(None);
        }

        let expr_result = self.parse_general_expression(1, false, true);
        let expr = match expr_result {
            Ok(e) => Some(e),
            Err(e) => {
                if e.is_empty_expression() {
                    None
                } else {
                    return Err(e);
                }
            }
        };

        let (tok_end, lit_end, pos_end) = self.scan_ignore_whitespace();
        let range_end = pos_end + lit_end.len();

        if tok_end != Token::BracketRight {
            return Err(ParseError::syntax(
                "expected ']' following expression",
                self.make_pos(pos_start, range_end),
            ));
        }

        if expr.is_none() {
            let expression_pos = self.make_pos(pos_start, range_end);
            let depth = self.child_expression_depth([1], expression_pos)?;
            return Ok(Some(Parsed {
                node: Expr::ArrayTraversal(ArrayTraversal {
                    pos: expression_pos,
                    expr: Box::new(Expr::Everything(Everything {
                        pos: self.make_pos(0, 0),
                    })),
                }),
                depth,
            }));
        }
        let expr = expr.unwrap();

        if let Expr::Literal(Literal::String(s)) = &expr.node {
            return Ok(Some(Parsed {
                node: Expr::Attribute(Attribute {
                    pos: self.make_pos(pos_start, range_end),
                    name: s.value.clone(),
                }),
                depth: 1,
            }));
        }

        if let Expr::Range(_) = &expr.node {
            if !expr.node.is_subscript_expression() {
                return Err(ParseError::syntax(
                    "subscript ranges must have integer endpoints",
                    self.make_pos(pos_start, range_end),
                ));
            }
            let expression_pos = self.make_pos(pos_start, range_end);
            let depth = self.child_expression_depth([expr.depth], expression_pos)?;
            return Ok(Some(Parsed {
                node: Expr::Subscript(Subscript {
                    pos: expression_pos,
                    value: Box::new(expr.node),
                }),
                depth,
            }));
        }

        if expr.node.is_subscript_expression() {
            let expression_pos = self.make_pos(pos_start, range_end);
            let depth = self.child_expression_depth([expr.depth], expression_pos)?;
            return Ok(Some(Parsed {
                node: Expr::Subscript(Subscript {
                    pos: expression_pos,
                    value: Box::new(expr.node),
                }),
                depth,
            }));
        }

        let expression_pos = self.make_pos(pos_start, range_end);
        let depth = self.child_expression_depth([expr.depth], expression_pos)?;
        Ok(Some(Parsed {
            node: Expr::Constraint(Constraint {
                pos: expression_pos,
                expression: Box::new(expr.node),
            }),
            depth,
        }))
    }

    fn parse_atom(&mut self, immediate_lhs: bool) -> Result<Option<Parsed<Expr>>, ParseError> {
        let (tok, lit, pos) = self.scan_ignore_whitespace();
        match tok {
            Token::Name => {
                if let Some(stripped) = lit.strip_prefix('$') {
                    let param_pos = self.make_token_pos(pos, lit);
                    // Track this parameter reference for validation
                    self.referenced_params
                        .push((stripped.to_string(), param_pos));
                    return Ok(Some(Parsed {
                        node: Expr::Param(Param {
                            pos: param_pos,
                            name: stripped.to_string(),
                        }),
                        depth: 1,
                    }));
                }

                if let Some(func) = self.parse_function_expression(tok, lit, pos)? {
                    return Ok(Some(Parsed {
                        node: Expr::FunctionCall(func.node),
                        depth: func.depth,
                    }));
                }

                Ok(Some(Parsed {
                    node: Expr::Attribute(Attribute {
                        pos: self.make_token_pos(pos, lit),
                        name: lit.to_string(),
                    }),
                    depth: 1,
                }))
            }
            Token::Asterisk => Ok(Some(Parsed {
                node: Expr::Everything(Everything {
                    pos: self.make_token_pos(pos, lit),
                }),
                depth: 1,
            })),
            Token::At => Ok(Some(Parsed {
                node: Expr::This(This {
                    pos: self.make_token_pos(pos, lit),
                }),
                depth: 1,
            })),
            Token::Hat => Ok(Some(Parsed {
                node: Expr::Parent(Parent {
                    pos: self.make_token_pos(pos, lit),
                }),
                depth: 1,
            })),
            Token::AscOperator | Token::DescOperator | Token::InOperator | Token::MatchOperator => {
                Ok(Some(Parsed {
                    node: Expr::Attribute(Attribute {
                        pos: self.make_token_pos(pos, lit),
                        name: lit.to_string(),
                    }),
                    depth: 1,
                }))
            }
            Token::Integer => {
                // Try parsing as i64 first, fall back to f64 for large numbers
                if let Ok(value) = lit.parse::<i64>() {
                    Ok(Some(Parsed {
                        node: Expr::Literal(Literal::Integer(IntegerLiteral {
                            pos: self.make_token_pos(pos, lit),
                            value,
                        })),
                        depth: 1,
                    }))
                } else {
                    // Integer too large for i64, parse as float
                    Ok(Some(Parsed {
                        node: Expr::Literal(Literal::Float(FloatLiteral {
                            pos: self.make_token_pos(pos, lit),
                            value: lit.parse::<f64>().map_err(|error| {
                                ParseError::syntax(
                                    format!("invalid number: {error}"),
                                    self.make_token_pos(pos, lit),
                                )
                            })?,
                        })),
                        depth: 1,
                    }))
                }
            }
            Token::Float => Ok(Some(Parsed {
                node: Expr::Literal(Literal::Float(FloatLiteral {
                    pos: self.make_token_pos(pos, lit),
                    value: lit.parse::<f64>().map_err(|error| {
                        ParseError::syntax(
                            format!("invalid number: {error}"),
                            self.make_token_pos(pos, lit),
                        )
                    })?,
                })),
                depth: 1,
            })),
            Token::String => {
                // Parse the string value, handling escape sequences
                let inner = &lit[1..lit.len() - 1];
                let value = unescape_string(inner);
                Ok(Some(Parsed {
                    node: Expr::Literal(Literal::String(StringLiteral {
                        pos: self.make_token_pos(pos, lit),
                        value,
                    })),
                    depth: 1,
                }))
            }
            Token::Bool => Ok(Some(Parsed {
                node: Expr::Literal(Literal::Boolean(BooleanLiteral {
                    pos: self.make_token_pos(pos, lit),
                    value: lit == "true",
                })),
                depth: 1,
            })),
            Token::Null => Ok(Some(Parsed {
                node: Expr::Literal(Literal::Null(NullLiteral {
                    pos: self.make_token_pos(pos, lit),
                })),
                depth: 1,
            })),
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
    ) -> Result<Parsed<Expr>, ParseError> {
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
                return Err(ParseError::syntax(
                    "expected ')' following parenthesized expression",
                    self.make_pos(pos, tok_pos),
                ));
            }

            let expression_pos = self.make_token_pos(pos, lit);
            let depth = self.child_expression_depth(
                tuple_members.iter().map(|member| member.depth),
                expression_pos,
            )?;
            return Ok(Parsed {
                node: Expr::Tuple(Tuple {
                    pos: expression_pos,
                    members: tuple_members
                        .into_iter()
                        .map(|member| member.node)
                        .collect(),
                }),
                depth,
            });
        }

        if tok != Token::ParenRight {
            return Err(ParseError::syntax(
                "expected ')' following parenthesized expression",
                self.make_pos(pos, tok_pos),
            ));
        }

        let expression_pos = self.make_token_pos(pos, lit);
        let depth = self.child_expression_depth([expr.depth], expression_pos)?;
        Ok(Parsed {
            node: Expr::Group(Group {
                pos: expression_pos,
                expression: Box::new(expr.node),
            }),
            depth,
        })
    }

    fn parse_array_expression(&mut self) -> Result<Parsed<Expr>, ParseError> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BracketLeft {
            return Err(ParseError::syntax(
                "expected '['",
                self.make_pos(pos_start, pos_start),
            ));
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
            return Err(ParseError::syntax(
                "expected ']' following array body",
                self.make_pos(pos_start, range_end),
            ));
        }

        let expression_pos = self.make_pos(pos_start, range_end);
        let depth = self.child_expression_depth(
            exprs.iter().map(|expression| expression.depth),
            expression_pos,
        )?;
        Ok(Parsed {
            node: Expr::Array(Array {
                pos: expression_pos,
                expressions: exprs
                    .into_iter()
                    .map(|expression| expression.node)
                    .collect(),
            }),
            depth,
        })
    }

    fn parse_object_expression(&mut self) -> Result<Parsed<Expr>, ParseError> {
        let (tok, _, pos_start) = self.scan_ignore_whitespace();
        if tok != Token::BraceLeft {
            return Err(ParseError::syntax(
                "expected '{'",
                self.make_pos(pos_start, pos_start),
            ));
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
            return Err(ParseError::syntax(
                "expected '}' following object body",
                self.make_pos(pos_start, range_end),
            ));
        }

        let expression_pos = self.make_pos(pos_start, range_end);
        let depth = self.child_expression_depth(
            exprs.iter().map(|expression| expression.depth),
            expression_pos,
        )?;
        Ok(Parsed {
            node: Expr::Object(Object {
                pos: expression_pos,
                expressions: exprs
                    .into_iter()
                    .map(|expression| expression.node)
                    .collect(),
            }),
            depth,
        })
    }

    fn parse_function_expression(
        &mut self,
        name_token: Token,
        name: &'a str,
        name_pos: usize,
    ) -> Result<Option<Parsed<FunctionCall>>, ParseError> {
        let (tok, lit, pos) = self.scan();

        match tok {
            Token::DoubleColon => {
                if name_token != Token::Name {
                    return Err(ParseError::syntax(
                        "expected valid namespace identifier before '::'",
                        self.make_token_pos(pos, lit),
                    ));
                }

                let (f_tok, f_lit, f_pos) = self.scan_ignore_whitespace();
                if f_tok != Token::Name {
                    return Err(ParseError::syntax(
                        "expected a function following namespace expression",
                        self.make_token_pos(f_pos, f_lit),
                    ));
                }

                let func_opt = self.parse_function_expression(Token::DoubleColon, f_lit, f_pos)?;

                return if let Some(mut func) = func_opt {
                    func.node.namespace = name.to_string();
                    func.node.pos.start = name_pos;
                    Ok(Some(func))
                } else {
                    Err(ParseError::syntax(
                        "expected a function following namespace expression",
                        self.make_token_pos(f_pos, f_lit),
                    ))
                };
            }
            Token::ParenLeft => {
                // Fall through to parse arguments
            }
            Token::Whitespace | Token::Comment => {
                if tok == Token::Comment && self.config.preserve_comments {
                    self.comments.push(Comment {
                        pos: Position {
                            start: pos,
                            end: pos + lit.len(),
                        },
                        text: lit.to_string(),
                    });
                }
                let (next_tok, _, _) = self.scan_ignore_whitespace();
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
            return Err(ParseError::syntax(
                "expected ')' following function arguments",
                self.make_pos(pos, range_end),
            ));
        }

        let expression_pos = self.make_token_pos(name_pos, name);
        let depth = self.child_expression_depth(
            exprs.iter().map(|expression| expression.depth),
            expression_pos,
        )?;
        Ok(Some(Parsed {
            node: FunctionCall {
                namespace: String::new(),
                pos: expression_pos,
                name: name.to_string(),
                arguments: exprs
                    .into_iter()
                    .map(|expression| expression.node)
                    .collect(),
            },
            depth,
        }))
    }
}

fn unescape_string(s: &str) -> String {
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

    result
}
