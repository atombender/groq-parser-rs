#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Token {
    Illegal,
    EOF,
    Whitespace,
    ParenLeft,
    ParenRight,
    BraceLeft,
    BraceRight,
    BracketLeft,
    BracketRight,
    Dot,
    Equals,
    Or,
    And,
    GT,
    LT,
    GTE,
    LTE,
    NEQ,
    Not,
    Null,
    Arrow,
    At,
    Comma,
    Colon,
    Name,
    DotDot,
    DotDotDot,
    MatchOperator,
    InOperator,
    AscOperator,
    DescOperator,
    Integer,
    Float,
    Bool,
    String,
    Hat,
    Asterisk,
    Exponentiation,
    Slash,
    Percent,
    Plus,
    Minus,
    Pipe,
    Rocket,
    DoubleColon,
    EqualSign,
    Semicolon,
}

impl Token {
    pub fn literal(&self) -> &'static str {
        match self {
            Token::Illegal => "<illegal>",
            Token::EOF => "<eof>",
            Token::Whitespace => " ",
            Token::ParenLeft => "(",
            Token::ParenRight => ")",
            Token::BraceLeft => "{",
            Token::BraceRight => "}",
            Token::BracketLeft => "[",
            Token::BracketRight => "]",
            Token::Dot => ".",
            Token::Equals => "==",
            Token::EqualSign => "=",
            Token::Arrow => "->",
            Token::At => "@",
            Token::Or => "||",
            Token::And => "&&",
            Token::GT => ">",
            Token::LT => "<",
            Token::GTE => ">=",
            Token::LTE => "<=",
            Token::NEQ => "!=",
            Token::Not => "!",
            Token::Comma => ",",
            Token::Colon => ":",
            Token::DoubleColon => "::",
            Token::Name => "<identifier>",
            Token::DotDotDot => "...",
            Token::DotDot => "..",
            Token::MatchOperator => "match",
            Token::InOperator => "in",
            Token::AscOperator => "asc",
            Token::DescOperator => "desc",
            Token::Integer => "integer",
            Token::Float => "float",
            Token::String => "string",
            Token::Bool => "bool",
            Token::Null => "null",
            Token::Hat => "^",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Asterisk => "*",
            Token::Exponentiation => "**",
            Token::Slash => "/",
            Token::Percent => "%",
            Token::Pipe => "|",
            Token::Rocket => "=>",
            Token::Semicolon => ";",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Everything(Everything),
    This(This),
    Parent(Parent),
    Constraint(Constraint),
    Object(Object),
    Array(Array),
    Subscript(Subscript),
    Range(Range),
    FunctionCall(FunctionCall),
    Binary(BinaryOperator),
    Dot(DotOperator),
    ArrayTraversal(ArrayTraversal),
    Group(Group),
    Tuple(Tuple),
    Pipe(PipeOperator),
    Prefix(PrefixOperator),
    Postfix(PostfixOperator),
    Attribute(Attribute),
    Param(Param),
    Literal(Literal),
    Projection(Projection),
    FunctionPipe(FunctionPipe),
    Filter(Filter),
    Element(Element),
    Slice(Slice),
    Ellipsis(Ellipsis),
}

impl Expr {
    pub fn is_subscript_expression(&self) -> bool {
        match self {
            Expr::Range(r) => r.start.is_subscript_expression() && r.end.is_subscript_expression(),
            Expr::Literal(Literal::Integer(_)) => true,
            Expr::Literal(Literal::Float(_)) => true,
            Expr::Binary(b) => {
                crate::parser::operators::is_arithmetic_operator(b.operator)
                    && b.lhs.is_subscript_expression()
                    && b.rhs.is_subscript_expression()
            }
            Expr::Group(g) => g.expression.is_subscript_expression(),
            Expr::Prefix(p) => {
                (p.operator == Token::Plus || p.operator == Token::Minus)
                    && p.rhs.is_subscript_expression()
            }
            Expr::Param(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(StringLiteral),
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Boolean(BooleanLiteral),
    Null(NullLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Everything {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct This {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parent {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub pos: Position,
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub pos: Position,
    pub expressions: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub pos: Position,
    pub expressions: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subscript {
    pub pos: Position,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub pos: Position,
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub namespace: String,
    pub pos: Position,
    pub name: String,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperator {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub operator: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DotOperator {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayTraversal {
    pub pos: Position,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub pos: Position,
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub pos: Position,
    pub members: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipeOperator {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixOperator {
    pub pos: Position,
    pub rhs: Box<Expr>,
    pub operator: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostfixOperator {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub operator: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub pos: Position,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub pos: Position,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub pos: Position,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullLiteral {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerLiteral {
    pub pos: Position,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteral {
    pub pos: Position,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteral {
    pub pos: Position,
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ellipsis {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub object: Object,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionPipe {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub func: FunctionCall,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub idx: Subscript,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub pos: Position,
    pub lhs: Box<Expr>,
    pub range: Subscript,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FunctionID {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub pos: Position,
    pub id: FunctionID,
    pub body: Expr,
    pub parameters: Vec<FunctionParamDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParamDefinition {
    pub index: usize,
    pub name: String,
}
