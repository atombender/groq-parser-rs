#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

/// Trait for AST nodes that have source position information.
pub trait HasPosition {
    fn get_pos(&self) -> Position;
}

/// Macro to implement HasPosition for structs with a `pos` field.
macro_rules! impl_has_position {
    ($($t:ty),+ $(,)?) => {
        $(
            impl HasPosition for $t {
                fn get_pos(&self) -> Position {
                    self.pos
                }
            }
        )+
    };
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
    Comment,
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
            Token::Comment => "// ...",
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

/// A comment found in the source query.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub pos: Position,
    pub text: String,
}

/// The result of parsing a GROQ query, containing both the main expression
/// and any custom function definitions.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub expr: Expr,
    pub functions: Vec<FunctionDefinition>,
    pub comments: Vec<Comment>,
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

// Implement HasPosition for all struct types with a `pos` field
impl_has_position!(
    Everything,
    This,
    Parent,
    Constraint,
    Object,
    Array,
    Subscript,
    Range,
    FunctionCall,
    BinaryOperator,
    DotOperator,
    ArrayTraversal,
    Group,
    Tuple,
    PipeOperator,
    PrefixOperator,
    PostfixOperator,
    Attribute,
    Param,
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    BooleanLiteral,
    NullLiteral,
    Ellipsis,
    Projection,
    FunctionPipe,
    Filter,
    Element,
    Slice,
    FunctionDefinition,
);

impl HasPosition for Literal {
    fn get_pos(&self) -> Position {
        match self {
            Literal::String(lit) => lit.get_pos(),
            Literal::Integer(lit) => lit.get_pos(),
            Literal::Float(lit) => lit.get_pos(),
            Literal::Boolean(lit) => lit.get_pos(),
            Literal::Null(lit) => lit.get_pos(),
        }
    }
}

impl HasPosition for Expr {
    fn get_pos(&self) -> Position {
        match self {
            Expr::Everything(node) => node.get_pos(),
            Expr::This(node) => node.get_pos(),
            Expr::Parent(node) => node.get_pos(),
            Expr::Constraint(node) => node.get_pos(),
            Expr::Object(node) => node.get_pos(),
            Expr::Array(node) => node.get_pos(),
            Expr::Subscript(node) => node.get_pos(),
            Expr::Range(node) => node.get_pos(),
            Expr::FunctionCall(node) => node.get_pos(),
            Expr::Binary(node) => node.get_pos(),
            Expr::Dot(node) => node.get_pos(),
            Expr::ArrayTraversal(node) => node.get_pos(),
            Expr::Group(node) => node.get_pos(),
            Expr::Tuple(node) => node.get_pos(),
            Expr::Pipe(node) => node.get_pos(),
            Expr::Prefix(node) => node.get_pos(),
            Expr::Postfix(node) => node.get_pos(),
            Expr::Attribute(node) => node.get_pos(),
            Expr::Param(node) => node.get_pos(),
            Expr::Literal(lit) => lit.get_pos(),
            Expr::Projection(node) => node.get_pos(),
            Expr::FunctionPipe(node) => node.get_pos(),
            Expr::Filter(node) => node.get_pos(),
            Expr::Element(node) => node.get_pos(),
            Expr::Slice(node) => node.get_pos(),
            Expr::Ellipsis(node) => node.get_pos(),
        }
    }
}
