use crate::ast::*;
use crate::parser::Parser;

// Helper to parse and unwrap
fn parse(src: &str) -> Expr {
    let mut parser = Parser::new(src);
    parser
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", src, e))
        .expr
}

// Helper to assert parse succeeds
fn assert_parses(src: &str) {
    let mut parser = Parser::new(src);
    assert!(parser.parse().is_ok(), "Failed to parse: {}", src);
}

// ==================== LITERALS ====================

#[test]
fn test_integer_literals() {
    assert_parses("0");
    assert_parses("1");
    assert_parses("42");
    assert_parses("123456789");
    // Large integers (parsed as float)
    assert_parses("9223372036854775808");
    assert_parses("99999999999999999999");
}

#[test]
fn test_float_literals() {
    assert_parses("0.0");
    assert_parses("3.14");
    assert_parses("0.123");
    assert_parses("123.456");
    // Scientific notation
    assert_parses("1e10");
    assert_parses("1E10");
    assert_parses("1.5e10");
    assert_parses("1.5E10");
    assert_parses("1e+10");
    assert_parses("1e-10");
    assert_parses("1.5e+10");
    assert_parses("1.5e-10");
    assert_parses("3.14e100");
}

#[test]
fn test_string_literals() {
    assert_parses(r#""hello""#);
    assert_parses(r#"'hello'"#);
    assert_parses(r#""""#); // empty string
    assert_parses(r#"''"#); // empty string single quotes
    assert_parses(r#""hello world""#);
}

#[test]
fn test_string_escape_sequences() {
    assert_parses(r#""hello\nworld""#); // newline
    assert_parses(r#""hello\tworld""#); // tab
    assert_parses(r#""hello\\world""#); // backslash
    assert_parses(r#""hello\"world""#); // quote
    assert_parses(r#""hello\'world""#); // single quote
    assert_parses(r#""hello\/world""#); // forward slash
    assert_parses(r#""hello\rworld""#); // carriage return
    assert_parses(r#""hello\bworld""#); // backspace
    assert_parses(r#""hello\fworld""#); // form feed
}

#[test]
fn test_string_unicode_escapes() {
    assert_parses(r#""\u0041""#); // A
    assert_parses(r#""\u00e9""#); // é
    assert_parses(r#""\u{1F600}""#); // emoji
    assert_parses(r#""\u{41}""#); // A (short form)
    // Surrogate pairs
    assert_parses(r#""\uD83D\uDE00""#); // emoji via surrogate pair
}

#[test]
fn test_boolean_literals() {
    assert_parses("true");
    assert_parses("false");
}

#[test]
fn test_null_literal() {
    assert_parses("null");
}

// ==================== SPECIAL EXPRESSIONS ====================

#[test]
fn test_everything() {
    assert_parses("*");
}

#[test]
fn test_this() {
    assert_parses("@");
}

#[test]
fn test_parent() {
    assert_parses("^");
}

// ==================== ATTRIBUTES & PARAMS ====================

#[test]
fn test_attributes() {
    assert_parses("foo");
    assert_parses("_type");
    assert_parses("_id");
    assert_parses("camelCase");
    assert_parses("snake_case");
    assert_parses("PascalCase");
    assert_parses("_private");
    assert_parses("a1");
    assert_parses("_123");
}

#[test]
fn test_params() {
    assert_parses("$param");
    assert_parses("$a");
    assert_parses("$myParam");
    assert_parses("$_private");
}

// ==================== ARRAYS ====================

#[test]
fn test_array_literals() {
    assert_parses("[]");
    assert_parses("[1]");
    assert_parses("[1, 2, 3]");
    assert_parses("[1, 2, 3,]"); // trailing comma
    assert_parses(r#"["a", "b", "c"]"#);
    assert_parses("[true, false, null]");
    assert_parses("[[1, 2], [3, 4]]"); // nested
    assert_parses("[1, [2, [3]]]"); // deeply nested
}

// ==================== OBJECTS ====================

#[test]
fn test_object_literals() {
    assert_parses("{}");
    assert_parses("{a}");
    assert_parses("{a, b}");
    assert_parses("{a, b,}"); // trailing comma
    assert_parses(r#"{"key": value}"#);
    assert_parses("{a: 1, b: 2}");
    assert_parses("{{nested}}");
}

// ==================== BINARY OPERATORS ====================

#[test]
fn test_comparison_operators() {
    assert_parses("a == b");
    assert_parses("a != b");
    assert_parses("a < b");
    assert_parses("a > b");
    assert_parses("a <= b");
    assert_parses("a >= b");
}

#[test]
fn test_logical_operators() {
    assert_parses("a && b");
    assert_parses("a || b");
    assert_parses("a && b && c");
    assert_parses("a || b || c");
    assert_parses("a && b || c");
    assert_parses("(a && b) || (c && d)");
}

#[test]
fn test_arithmetic_operators() {
    assert_parses("a + b");
    assert_parses("a - b");
    assert_parses("a * b");
    assert_parses("a / b");
    assert_parses("a % b");
    assert_parses("a ** b"); // exponentiation
    assert_parses("a + b * c");
    assert_parses("(a + b) * c");
    assert_parses("a ** b ** c"); // right associative
}

#[test]
fn test_in_operator() {
    assert_parses("a in b");
    assert_parses(r#""foo" in tags"#);
    assert_parses("x in [1, 2, 3]");
}

#[test]
fn test_match_operator() {
    assert_parses("a match b");
    assert_parses(r#"title match "foo*""#);
}

#[test]
fn test_range_operators() {
    assert_parses("0..10"); // exclusive
    assert_parses("0...10"); // inclusive
    assert_parses("a..b");
    assert_parses("a...b");
    assert_parses("0..count(*)");
}

#[test]
fn test_rocket_operator() {
    assert_parses("key => value");
    assert_parses(r#""name" => title"#);
}

#[test]
fn test_colon_operator() {
    assert_parses("a: b");
    assert_parses(r#""key": value"#);
}

// ==================== PREFIX OPERATORS ====================

#[test]
fn test_prefix_operators() {
    assert_parses("!a");
    assert_parses("!true");
    assert_parses("!!a");
    assert_parses("+1");
    assert_parses("-1");
    assert_parses("-a");
    assert_parses("--a");
    assert_parses("+-a");
}

#[test]
fn test_spread_operator() {
    assert_parses("...a");
    assert_parses("{...a}");
    assert_parses("{...a, b}");
    assert_parses("{a, ...b}");
    assert_parses("[...a]");
}

#[test]
fn test_ellipsis() {
    assert_parses("{...}");
    assert_parses("{a, ...}");
}

// ==================== POSTFIX OPERATORS ====================

#[test]
fn test_dereference_operator() {
    assert_parses("ref->");
    assert_parses("a.ref->");
    assert_parses("a->b");
    assert_parses("a->b->c");
}

#[test]
fn test_ordering_operators() {
    assert_parses("title asc");
    assert_parses("title desc");
    assert_parses("a.b asc");
    assert_parses("a.b desc");
}

// ==================== DOT ACCESS ====================

#[test]
fn test_dot_access() {
    assert_parses("a.b");
    assert_parses("a.b.c");
    assert_parses("a.b.c.d");
    assert_parses("*.a");
    assert_parses("@.a");
    assert_parses("^.a");
}

#[test]
fn test_bracket_attribute_access() {
    assert_parses(r#"a["key"]"#);
    assert_parses(r#"a['key']"#);
    assert_parses(r#"a["key"]["nested"]"#);
}

// ==================== ELEMENT ACCESS ====================

#[test]
fn test_element_access() {
    assert_parses("a[0]");
    assert_parses("a[1]");
    assert_parses("a[-1]");
    assert_parses("a[0][1]");
    assert_parses("a[$idx]");
    assert_parses("a[b + c]");
    // Grouped subscript
    assert_parses("a[(0)]");
    assert_parses("a[(1 + 2)]");
    // Arithmetic subscript with integer operands
    assert_parses("a[1 + 2]");
    assert_parses("a[1 - 2]");
    assert_parses("a[2 * 3]");
    assert_parses("a[10 / 2]");
    assert_parses("a[10 % 3]");
}

// ==================== SLICE ACCESS ====================

#[test]
fn test_slice_access() {
    assert_parses("a[0..10]");
    assert_parses("a[0...10]");
    assert_parses("a[$start..$end]");
    assert_parses("a[0..$n]");
    assert_parses("a[$start..10]");
    // Float endpoints in ranges
    assert_parses("a[0.0..10.0]");
    assert_parses("a[0..1.5]");
    assert_parses("a[0.5..10]");
    // Note: a[0..count(*)] is invalid - slice ranges must have integer endpoints
}

// ==================== FILTER ====================

#[test]
fn test_filter() {
    assert_parses("*[true]");
    assert_parses("*[_type == \"movie\"]");
    assert_parses("*[a && b]");
    assert_parses("*[a || b]");
    assert_parses("a[@ > 0]");
    assert_parses("*[_type == \"movie\"][0..10]");
}

// ==================== ARRAY TRAVERSAL ====================

#[test]
fn test_array_traversal() {
    assert_parses("a[]");
    assert_parses("a[].b");
    assert_parses("a[].b[]");
    assert_parses("*[].tags[]");
}

// ==================== PROJECTION ====================

#[test]
fn test_projection() {
    assert_parses("*{}");
    assert_parses("*{a}");
    assert_parses("*{a, b}");
    assert_parses("*{a, b, c}");
    assert_parses(r#"*{"key": a}"#);
    assert_parses("*{...}");
    assert_parses("*{..., a}");
    assert_parses("*[_type == \"movie\"]{title}");
    assert_parses("*[_type == \"movie\"]{title, year}");
}

#[test]
fn test_nested_projection() {
    assert_parses("*{a{b}}");
    assert_parses("*{a{b, c}}");
    assert_parses("*{a[]{b}}");
}

// ==================== FUNCTION CALLS ====================

#[test]
fn test_function_calls() {
    assert_parses("count()");
    assert_parses("count(*)");
    assert_parses("count(*[])");
    assert_parses("length(a)");
    assert_parses("round(3.14)");
    assert_parses("round(3.14, 1)");
    assert_parses("coalesce(a, b)");
    assert_parses("coalesce(a, b, c)");
    assert_parses("defined(a)");
    assert_parses("references(a)");
    assert_parses("now()");
    assert_parses("lower(title)");
    assert_parses("upper(title)");
    assert_parses("string(123)");
    assert_parses("path(a)");
    assert_parses("select(a => 1, b => 2)");
}

#[test]
fn test_namespaced_function_calls() {
    assert_parses("global::now()");
    assert_parses("string::lower(a)");
    assert_parses("string::upper(a)");
    assert_parses("math::round(a)");
    assert_parses("math::sum(a)");
    assert_parses("array::join(a, b)");
    assert_parses("pt::text(a)");
    assert_parses("geo::distance(a, b)");
    assert_parses("dateTime::now()");
    assert_parses("sanity::projectId()");
}

// ==================== FUNCTION PIPE ====================

#[test]
fn test_function_pipe() {
    assert_parses("* | count()");
    assert_parses("*[] | count()");
    assert_parses("*[_type == \"movie\"] | count()");
    assert_parses("* | order(title asc)");
    assert_parses("* | order(title asc) | count()");
    assert_parses("* | score(title match $q)");
}

// ==================== GROUPING ====================

#[test]
fn test_grouping() {
    assert_parses("(a)");
    assert_parses("(a + b)");
    assert_parses("(a + b) * c");
    assert_parses("a * (b + c)");
    assert_parses("((a))");
    assert_parses("((a + b))");
}

// ==================== TUPLES ====================

#[test]
fn test_tuples() {
    assert_parses("(a, b)");
    assert_parses("(a, b, c)");
    assert_parses("(1, 2, 3)");
    assert_parses("(a, b, c, d)");
}

// ==================== FUNCTION DEFINITIONS ====================

#[test]
fn test_function_definitions() {
    assert_parses("fn my::func($a) = $a; *");
    assert_parses("fn ns::identity($x) = $x; count(*)");
    assert_parses("fn math::double($n) = $n * 2; *");
}

#[test]
fn test_parse_custom_function() {
    let mut parser = Parser::new(
        "fn pt::isPublished($doc) = !(_id in path('drafts.**')); *[pt::isPublished(@)]",
    );
    let result = parser.parse().unwrap();
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].id.namespace, "pt");
    assert_eq!(result.functions[0].id.name, "isPublished");
    assert_eq!(result.functions[0].parameters.len(), 1);
    assert_eq!(result.functions[0].parameters[0].name, "doc");
}

#[test]
fn test_parse_multiple_function_definitions() {
    let mut parser = Parser::new("fn math::double($n) = $n * 2; fn math::triple($n) = $n * 3; *");
    let result = parser.parse().unwrap();
    assert_eq!(result.functions.len(), 2);
    assert_eq!(result.functions[0].id.name, "double");
    assert_eq!(result.functions[1].id.name, "triple");
}

#[test]
fn test_parse_no_functions() {
    let mut parser = Parser::new("*[_type == \"movie\"]");
    let result = parser.parse().unwrap();
    assert_eq!(result.functions.len(), 0);
}

// ==================== COMMENTS ====================

#[test]
fn test_comments() {
    assert_parses("* // comment");
    assert_parses("// comment\n*");
    assert_parses("*[_type == \"movie\"] // filter\n{title}");
}

// ==================== COMPLEX QUERIES ====================

#[test]
fn test_complex_queries() {
    // Real-world style queries
    assert_parses(r#"*[_type == "movie" && releaseDate > "2020-01-01"]"#);
    assert_parses(r#"*[_type == "movie"]{title, "director": director->name}"#);
    assert_parses(r#"*[_type == "movie"] | order(releaseDate desc) [0...10]"#);
    assert_parses(r#"*[_type == "post" && author._ref == $authorId]{title, body}"#);
    assert_parses(r#"*[_type == "article" && references($id)]"#);
    assert_parses(r#"count(*[_type == "movie" && rating > 8])"#);
    assert_parses(r#"*[_type == "movie"]{..., "cast": castMembers[]->name}"#);
    assert_parses(
        r#"*[_type == "category"]{name, "products": *[_type == "product" && references(^._id)]}"#,
    );
}

#[test]
fn test_deeply_nested() {
    assert_parses("a.b.c.d.e.f.g");
    assert_parses("a[0][1][2][3]");
    assert_parses("a[b[c[d]]]");
    assert_parses("a{b{c{d{e}}}}");
    assert_parses("((((a))))");
}

#[test]
fn test_operator_precedence() {
    // These should parse without errors - precedence is handled correctly
    assert_parses("a + b * c");
    assert_parses("a * b + c");
    assert_parses("a && b || c");
    assert_parses("a || b && c");
    assert_parses("a == b && c == d");
    assert_parses("!a && b");
    assert_parses("a + b == c + d");
    assert_parses("-a + b");
    assert_parses("a ** b ** c");
}

#[test]
fn test_whitespace_handling() {
    // Various whitespace patterns
    assert_parses("  *  ");
    assert_parses("a   +   b");
    assert_parses("a\n+\nb");
    assert_parses("a\t+\tb");
    assert_parses("*[\n  _type == \"movie\"\n]");
    assert_parses("*\n{\n  title\n}");
}

// ==================== EDGE CASES ====================

#[test]
fn test_keywords_as_attributes() {
    // Keywords that can also be attribute names
    assert_parses("a.in");
    assert_parses("a.match");
    assert_parses("a.asc");
    assert_parses("a.desc");
    assert_parses("a.true");
    assert_parses("a.false");
    assert_parses("a.null");
}

#[test]
fn test_negative_numbers() {
    assert_parses("-1");
    assert_parses("-3.14");
    assert_parses("-1e10");
    assert_parses("a[-1]");
    assert_parses("a + -1");
}

#[test]
fn test_chained_operations() {
    assert_parses("*[_type == \"a\"][0..10]{title}");
    assert_parses("*[a][b][c]");
    assert_parses("a.b.c[0].d.e[1]");
    assert_parses("a->b->c->d");
    assert_parses("a[].b[].c[]");
}

// ==================== VERIFY AST STRUCTURE ====================

#[test]
fn test_ast_everything() {
    let expr = parse("*");
    assert!(matches!(expr, Expr::Everything(_)));
}

#[test]
fn test_ast_this() {
    let expr = parse("@");
    assert!(matches!(expr, Expr::This(_)));
}

#[test]
fn test_ast_parent() {
    let expr = parse("^");
    assert!(matches!(expr, Expr::Parent(_)));
}

#[test]
fn test_ast_integer() {
    let expr = parse("42");
    match expr {
        Expr::Literal(Literal::Integer(i)) => assert_eq!(i.value, 42),
        _ => panic!("Expected IntegerLiteral"),
    }
}

#[test]
fn test_ast_float() {
    let expr = parse("1.234");
    match expr {
        Expr::Literal(Literal::Float(f)) => assert!((f.value - 1.234).abs() < 0.001),
        _ => panic!("Expected FloatLiteral"),
    }
}

#[test]
fn test_ast_string() {
    let expr = parse(r#""hello""#);
    match expr {
        Expr::Literal(Literal::String(s)) => assert_eq!(s.value, "hello"),
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_ast_string_with_escapes() {
    let expr = parse(r#""hello\nworld""#);
    match expr {
        Expr::Literal(Literal::String(s)) => assert_eq!(s.value, "hello\nworld"),
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_ast_boolean() {
    let expr = parse("true");
    match expr {
        Expr::Literal(Literal::Boolean(b)) => assert!(b.value),
        _ => panic!("Expected BooleanLiteral"),
    }
}

#[test]
fn test_ast_null() {
    let expr = parse("null");
    assert!(matches!(expr, Expr::Literal(Literal::Null(_))));
}

#[test]
fn test_ast_attribute() {
    let expr = parse("foo");
    match expr {
        Expr::Attribute(a) => assert_eq!(a.name, "foo"),
        _ => panic!("Expected Attribute"),
    }
}

#[test]
fn test_ast_param() {
    let expr = parse("$myParam");
    match expr {
        Expr::Param(p) => assert_eq!(p.name, "myParam"),
        _ => panic!("Expected Param"),
    }
}

#[test]
fn test_ast_array() {
    let expr = parse("[1, 2, 3]");
    match expr {
        Expr::Array(a) => assert_eq!(a.expressions.len(), 3),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_ast_object() {
    let expr = parse("{a, b, c}");
    match expr {
        Expr::Object(o) => assert_eq!(o.expressions.len(), 3),
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_ast_binary() {
    let expr = parse("a + b");
    match expr {
        Expr::Binary(b) => {
            assert_eq!(b.operator, Token::Plus);
            assert!(matches!(*b.lhs, Expr::Attribute(_)));
            assert!(matches!(*b.rhs, Expr::Attribute(_)));
        }
        _ => panic!("Expected Binary"),
    }
}

#[test]
fn test_ast_filter() {
    let expr = parse("*[_type == \"movie\"]");
    match expr {
        Expr::Filter(f) => {
            assert!(matches!(*f.lhs, Expr::Everything(_)));
            assert!(matches!(*f.constraint.expression, Expr::Binary(_)));
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_ast_projection() {
    let expr = parse("*{title}");
    match expr {
        Expr::Projection(p) => {
            assert!(matches!(*p.lhs, Expr::Everything(_)));
            assert_eq!(p.object.expressions.len(), 1);
        }
        _ => panic!("Expected Projection"),
    }
}

#[test]
fn test_ast_function_call() {
    let expr = parse("count(*)");
    match expr {
        Expr::FunctionCall(f) => {
            assert_eq!(f.name, "count");
            assert_eq!(f.arguments.len(), 1);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_ast_namespaced_function() {
    let expr = parse("string::lower(a)");
    match expr {
        Expr::FunctionCall(f) => {
            assert_eq!(f.namespace, "string");
            assert_eq!(f.name, "lower");
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_ast_dot() {
    let expr = parse("a.b");
    match expr {
        Expr::Dot(d) => {
            assert!(matches!(*d.lhs, Expr::Attribute(_)));
            assert!(matches!(*d.rhs, Expr::Attribute(_)));
        }
        _ => panic!("Expected Dot"),
    }
}

#[test]
fn test_ast_array_traversal() {
    let expr = parse("a[]");
    assert!(matches!(expr, Expr::ArrayTraversal(_)));
}

#[test]
fn test_ast_element() {
    let expr = parse("a[0]");
    assert!(matches!(expr, Expr::Element(_)));
}

#[test]
fn test_ast_slice() {
    let expr = parse("a[0..10]");
    assert!(matches!(expr, Expr::Slice(_)));
}

#[test]
fn test_ast_group() {
    let expr = parse("(a)");
    assert!(matches!(expr, Expr::Group(_)));
}

#[test]
fn test_ast_tuple() {
    let expr = parse("(a, b)");
    match expr {
        Expr::Tuple(t) => assert_eq!(t.members.len(), 2),
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_ast_prefix() {
    let expr = parse("!a");
    match expr {
        Expr::Prefix(p) => {
            assert_eq!(p.operator, Token::Not);
        }
        _ => panic!("Expected Prefix"),
    }
}

#[test]
fn test_ast_postfix_arrow() {
    let expr = parse("a->");
    match expr {
        Expr::Postfix(p) => {
            assert_eq!(p.operator, Token::Arrow);
        }
        _ => panic!("Expected Postfix"),
    }
}

#[test]
fn test_ast_postfix_asc() {
    let expr = parse("a asc");
    match expr {
        Expr::Postfix(p) => {
            assert_eq!(p.operator, Token::AscOperator);
        }
        _ => panic!("Expected Postfix"),
    }
}

#[test]
fn test_ast_ellipsis() {
    let expr = parse("{...}");
    match expr {
        Expr::Object(o) => {
            assert_eq!(o.expressions.len(), 1);
            assert!(matches!(o.expressions[0], Expr::Ellipsis(_)));
        }
        _ => panic!("Expected Object with Ellipsis"),
    }
}

#[test]
fn test_ast_range() {
    // In GROQ: ".." is inclusive, "..." is exclusive
    let expr = parse("0..10");
    match expr {
        Expr::Range(r) => {
            assert!(r.inclusive);
        }
        _ => panic!("Expected Range"),
    }

    let expr = parse("0...10");
    match expr {
        Expr::Range(r) => {
            assert!(!r.inclusive);
        }
        _ => panic!("Expected Range"),
    }
}

#[test]
fn test_ast_function_pipe() {
    let expr = parse("* | count()");
    match expr {
        Expr::FunctionPipe(fp) => {
            assert!(matches!(*fp.lhs, Expr::Everything(_)));
            assert_eq!(fp.func.name, "count");
        }
        _ => panic!("Expected FunctionPipe"),
    }
}

// ==================== ERROR CASES ====================

// Helper to assert parse fails
fn assert_parse_fails(src: &str) {
    let mut parser = Parser::new(src);
    assert!(parser.parse().is_err(), "Expected parse to fail: {}", src);
}

#[test]
fn test_error_empty_query() {
    assert_parse_fails("");
    assert_parse_fails("   ");
    assert_parse_fails("\n\n");
}

#[test]
fn test_error_trailing_content() {
    // Multiple expressions without operator
    assert_parse_fails("a b");
    assert_parse_fails("1 2");
    assert_parse_fails("* *");
}

#[test]
fn test_error_function_definition_missing_paren() {
    assert_parse_fails("fn ns::func = $a; *");
}

#[test]
fn test_error_function_definition_unclosed_paren() {
    assert_parse_fails("fn ns::func($a = $a; *");
}

#[test]
fn test_error_function_definition_missing_equals() {
    assert_parse_fails("fn ns::func($a) $a; *");
}

#[test]
fn test_error_function_definition_missing_semicolon() {
    assert_parse_fails("fn ns::func($a) = $a *");
}

#[test]
fn test_error_function_definition_missing_namespace() {
    assert_parse_fails("fn ($a) = $a; *");
}

#[test]
fn test_error_function_definition_missing_double_colon() {
    assert_parse_fails("fn ns func($a) = $a; *");
}

#[test]
fn test_error_function_definition_missing_name() {
    assert_parse_fails("fn ns::($a) = $a; *");
}

#[test]
fn test_error_function_definition_invalid_param() {
    assert_parse_fails("fn ns::func(a) = a; *");
}

#[test]
fn test_error_unexpected_token() {
    assert_parse_fails(")");
    assert_parse_fails("}");
    assert_parse_fails("]");
    assert_parse_fails(",");
    assert_parse_fails("==");
}

#[test]
fn test_error_unclosed_paren() {
    assert_parse_fails("(a");
    assert_parse_fails("((a)");
}

#[test]
fn test_error_unclosed_bracket() {
    assert_parse_fails("[a");
    assert_parse_fails("a[0");
}

#[test]
fn test_error_unclosed_brace() {
    assert_parse_fails("{a");
    assert_parse_fails("*{a");
}

#[test]
fn test_error_pipe_with_invalid_rhs() {
    // Pipe must be followed by object or function
    assert_parse_fails("* | 123");
    assert_parse_fails("* | a");
}

#[test]
fn test_error_unterminated_string() {
    assert_parse_fails(r#""hello"#);
    assert_parse_fails(r#"'hello"#);
}

#[test]
fn test_error_unterminated_escape() {
    assert_parse_fails(r#""hello\"#);
}

#[test]
fn test_error_invalid_number() {
    // Trailing decimal point
    assert_parse_fails("123.");
    // Trailing exponent
    assert_parse_fails("123e");
    assert_parse_fails("123e+");
}

#[test]
fn test_error_too_many_dots() {
    assert_parse_fails("....");
    assert_parse_fails("a....b");
}
