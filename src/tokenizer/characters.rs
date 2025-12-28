pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

pub fn is_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

pub fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

pub fn is_identifier_character(ch: char) -> bool {
    is_letter(ch) || is_digit(ch) || ch == '_'
}

pub fn is_leading_identifier_character(ch: char) -> bool {
    is_letter(ch) || ch == '_' || ch == '$'
}

pub fn is_operator_character(ch: char) -> bool {
    matches!(
        ch,
        '<' | '>' | '|' | '=' | '&' | '!' | '*' | '/' | '+' | '-' | '%'
    )
}
