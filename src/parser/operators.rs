use crate::ast::Token;

pub const HIGHEST_PRECEDENCE: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

pub fn precedence_and_associativity(token: Token) -> (i32, Associativity) {
    match token {
        Token::Dot | Token::Pipe => (HIGHEST_PRECEDENCE, Associativity::Left),

        Token::Arrow | Token::AscOperator | Token::DescOperator => (11, Associativity::Left),

        Token::Exponentiation => (10, Associativity::Right),

        Token::Asterisk | Token::Slash | Token::Percent => (9, Associativity::Left),

        Token::Not => (7, Associativity::Right),
        Token::Plus | Token::Minus => (7, Associativity::Left),
        Token::DotDot | Token::DotDotDot => (6, Associativity::Left),

        Token::MatchOperator
        | Token::Equals
        | Token::EqualSign
        | Token::GT
        | Token::GTE
        | Token::InOperator
        | Token::LT
        | Token::LTE
        | Token::NEQ => (5, Associativity::Left),

        Token::And => (4, Associativity::Left),

        Token::Or => (3, Associativity::Left),

        Token::Rocket => (2, Associativity::Left),

        Token::Colon | Token::Comma | Token::DoubleColon | Token::Semicolon => {
            (1, Associativity::Left)
        }

        _ => (-1, Associativity::Left),
    }
}

pub fn is_prefix_operator(token: Token) -> bool {
    matches!(
        token,
        Token::Not | Token::Plus | Token::Minus | Token::DotDotDot
    )
}

pub fn is_infix_operator(token: Token) -> bool {
    matches!(
        token,
        Token::Equals
            | Token::Or
            | Token::And
            | Token::GT
            | Token::LT
            | Token::GTE
            | Token::LTE
            | Token::NEQ
            | Token::Colon
            | Token::Dot
            | Token::DotDot
            | Token::MatchOperator
            | Token::DotDotDot
            | Token::InOperator
            | Token::Pipe
            | Token::Rocket
            | Token::Plus
            | Token::Minus
            | Token::Asterisk
            | Token::Slash
            | Token::Exponentiation
            | Token::Percent
            | Token::DoubleColon
    )
}

pub fn is_postfix_operator(token: Token) -> bool {
    matches!(
        token,
        Token::AscOperator | Token::DescOperator | Token::Arrow
    )
}

pub fn is_operator(token: Token) -> bool {
    is_infix_operator(token) || is_prefix_operator(token) || is_postfix_operator(token)
}

pub fn is_arithmetic_operator(token: Token) -> bool {
    matches!(
        token,
        Token::Plus
            | Token::Minus
            | Token::Asterisk
            | Token::Slash
            | Token::Exponentiation
            | Token::Percent
    )
}
