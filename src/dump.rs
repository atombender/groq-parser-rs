use crate::ast::*;

/// Format a float for display, using scientific notation for very large/small values
/// Matches Go's formatting: explicit +/- sign, two-digit exponent
fn format_float(v: f64) -> String {
    if v == 0.0 || (v.abs() >= 1e-4 && v.abs() < 1e16) {
        // Use regular formatting for "normal" sized numbers
        let s = format!("{}", v);
        // Ensure there's a decimal point for floats
        if !s.contains('.') && !s.contains('e') {
            format!("{}.0", s)
        } else {
            s
        }
    } else {
        // Use scientific notation for very large or very small numbers
        // Format like Go: e+XX or e-XX with at least 2 digit exponent
        let s = format!("{:e}", v);
        // Fix exponent format to match Go (e+02 instead of e2, e-05 instead of e-5)
        if let Some(e_pos) = s.find('e') {
            let (mantissa, exp) = s.split_at(e_pos);
            let exp_part = &exp[1..]; // skip 'e'
            let (sign, digits) = if let Some(stripped) = exp_part.strip_prefix('-') {
                ("-", stripped)
            } else if let Some(stripped) = exp_part.strip_prefix('+') {
                ("+", stripped)
            } else {
                ("+", exp_part)
            };
            format!("{}e{}{:0>2}", mantissa, sign, digits)
        } else {
            s
        }
    }
}

/// Escape a string for display, using Go-compatible escape sequences
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000c}' => result.push_str("\\f"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

pub fn dump_expr(expr: &Expr) -> String {
    let mut output = String::new();
    dump_expr_inner(expr, 0, &mut output);
    output
}

fn indent(level: usize, output: &mut String) {
    for _ in 0..level {
        output.push_str("  ");
    }
}

fn dump_expr_inner(expr: &Expr, level: usize, output: &mut String) {
    match expr {
        Expr::Everything(_) => {
            indent(level, output);
            output.push_str("Everything\n");
        }
        Expr::This(_) => {
            indent(level, output);
            output.push_str("This\n");
        }
        Expr::Parent(_) => {
            indent(level, output);
            output.push_str("Parent\n");
        }
        Expr::Constraint(c) => {
            indent(level, output);
            output.push_str("Constraint\n");
            indent(level + 1, output);
            output.push_str("expression:\n");
            dump_expr_inner(&c.expression, level + 2, output);
        }
        Expr::Object(obj) => {
            indent(level, output);
            output.push_str("Object\n");
            for (i, e) in obj.expressions.iter().enumerate() {
                indent(level + 1, output);
                output.push_str(&format!("[{}]:\n", i));
                dump_expr_inner(e, level + 2, output);
            }
        }
        Expr::Array(arr) => {
            indent(level, output);
            output.push_str("Array\n");
            for (i, e) in arr.expressions.iter().enumerate() {
                indent(level + 1, output);
                output.push_str(&format!("[{}]:\n", i));
                dump_expr_inner(e, level + 2, output);
            }
        }
        Expr::Subscript(sub) => {
            indent(level, output);
            output.push_str("Subscript\n");
            indent(level + 1, output);
            output.push_str("value:\n");
            dump_expr_inner(&sub.value, level + 2, output);
        }
        Expr::Range(r) => {
            indent(level, output);
            let op = if r.inclusive { ".." } else { "..." };
            output.push_str(&format!("Range {}\n", op));
            indent(level + 1, output);
            output.push_str("start:\n");
            dump_expr_inner(&r.start, level + 2, output);
            indent(level + 1, output);
            output.push_str("end:\n");
            dump_expr_inner(&r.end, level + 2, output);
        }
        Expr::FunctionCall(f) => {
            indent(level, output);
            if f.namespace.is_empty() {
                output.push_str(&format!("FunctionCall {}\n", f.name));
            } else {
                output.push_str(&format!("FunctionCall {}::{}\n", f.namespace, f.name));
            }
            for (i, arg) in f.arguments.iter().enumerate() {
                indent(level + 1, output);
                output.push_str(&format!("arg[{}]:\n", i));
                dump_expr_inner(arg, level + 2, output);
            }
        }
        Expr::Binary(b) => {
            indent(level, output);
            output.push_str(&format!("Binary {}\n", token_symbol(b.operator)));
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&b.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("rhs:\n");
            dump_expr_inner(&b.rhs, level + 2, output);
        }
        Expr::Dot(d) => {
            indent(level, output);
            output.push_str("Dot\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&d.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("rhs:\n");
            dump_expr_inner(&d.rhs, level + 2, output);
        }
        Expr::ArrayTraversal(at) => {
            indent(level, output);
            output.push_str("ArrayTraversal\n");
            indent(level + 1, output);
            output.push_str("expr:\n");
            dump_expr_inner(&at.expr, level + 2, output);
        }
        Expr::Group(g) => {
            indent(level, output);
            output.push_str("Group\n");
            indent(level + 1, output);
            output.push_str("expression:\n");
            dump_expr_inner(&g.expression, level + 2, output);
        }
        Expr::Tuple(t) => {
            indent(level, output);
            output.push_str("Tuple\n");
            for (i, m) in t.members.iter().enumerate() {
                indent(level + 1, output);
                output.push_str(&format!("[{}]:\n", i));
                dump_expr_inner(m, level + 2, output);
            }
        }
        Expr::Pipe(p) => {
            indent(level, output);
            output.push_str("Pipe\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&p.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("rhs:\n");
            dump_expr_inner(&p.rhs, level + 2, output);
        }
        Expr::Prefix(p) => {
            indent(level, output);
            output.push_str(&format!("Prefix {}\n", token_symbol(p.operator)));
            indent(level + 1, output);
            output.push_str("rhs:\n");
            dump_expr_inner(&p.rhs, level + 2, output);
        }
        Expr::Postfix(p) => {
            indent(level, output);
            output.push_str(&format!("Postfix {}\n", token_symbol(p.operator)));
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&p.lhs, level + 2, output);
        }
        Expr::Attribute(a) => {
            indent(level, output);
            output.push_str(&format!("Attribute {:?}\n", a.name));
        }
        Expr::Param(p) => {
            indent(level, output);
            output.push_str(&format!("Param {:?}\n", p.name));
        }
        Expr::Literal(lit) => match lit {
            Literal::String(s) => {
                indent(level, output);
                output.push_str(&format!("String {}\n", escape_string(&s.value)));
            }
            Literal::Integer(i) => {
                indent(level, output);
                output.push_str(&format!("Integer {}\n", i.value));
            }
            Literal::Float(f) => {
                indent(level, output);
                output.push_str(&format!("Float {}\n", format_float(f.value)));
            }
            Literal::Boolean(b) => {
                indent(level, output);
                output.push_str(&format!("Boolean {}\n", b.value));
            }
            Literal::Null(_) => {
                indent(level, output);
                output.push_str("Null\n");
            }
        },
        Expr::Projection(p) => {
            indent(level, output);
            output.push_str("Projection\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&p.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("object:\n");
            dump_expr_inner(&Expr::Object(p.object.clone()), level + 2, output);
        }
        Expr::FunctionPipe(fp) => {
            indent(level, output);
            output.push_str("FunctionPipe\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&fp.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("func:\n");
            dump_expr_inner(&Expr::FunctionCall(fp.func.clone()), level + 2, output);
        }
        Expr::Filter(f) => {
            indent(level, output);
            output.push_str("Filter\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&f.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("constraint:\n");
            dump_expr_inner(&Expr::Constraint(f.constraint.clone()), level + 2, output);
        }
        Expr::Element(e) => {
            indent(level, output);
            output.push_str("Element\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&e.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("idx:\n");
            dump_expr_inner(&Expr::Subscript(e.idx.clone()), level + 2, output);
        }
        Expr::Slice(s) => {
            indent(level, output);
            output.push_str("Slice\n");
            indent(level + 1, output);
            output.push_str("lhs:\n");
            dump_expr_inner(&s.lhs, level + 2, output);
            indent(level + 1, output);
            output.push_str("range:\n");
            dump_expr_inner(&Expr::Subscript(s.range.clone()), level + 2, output);
        }
        Expr::Ellipsis(_) => {
            indent(level, output);
            output.push_str("Ellipsis\n");
        }
    }
}

fn token_symbol(tok: Token) -> &'static str {
    match tok {
        Token::Equals => "==",
        Token::NEQ => "!=",
        Token::GT => ">",
        Token::LT => "<",
        Token::GTE => ">=",
        Token::LTE => "<=",
        Token::And => "&&",
        Token::Or => "||",
        Token::Not => "!",
        Token::Plus => "+",
        Token::Minus => "-",
        Token::Asterisk => "*",
        Token::Slash => "/",
        Token::Percent => "%",
        Token::Exponentiation => "**",
        Token::Pipe => "|",
        Token::Arrow => "->",
        Token::Dot => ".",
        Token::DotDot => "..",
        Token::DotDotDot => "...",
        Token::Colon => ":",
        Token::MatchOperator => "match",
        Token::InOperator => "in",
        Token::AscOperator => "asc",
        Token::DescOperator => "desc",
        Token::Rocket => "=>",
        _ => "<??>",
    }
}
