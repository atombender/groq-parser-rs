use groq_parser::dump::dump_expr;
use groq_parser::parser::{Parser, ParserConfig};
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Deserialize)]
struct TestEntry {
    #[serde(rename = "_type")]
    entry_type: String,
    #[serde(rename = "_id")]
    id: Option<String>,
    query: Option<String>,
    params: Option<Value>,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        let entry: TestEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Skip dataset entries, only process test entries
        if entry.entry_type != "test" {
            continue;
        }

        let query = match &entry.query {
            Some(q) => q,
            None => continue,
        };

        let id = entry.id.as_deref().unwrap_or("unknown");

        // Build parser config with params from test entry
        let config = match &entry.params {
            Some(Value::Object(params_obj)) => {
                // Extract param names from the object keys
                ParserConfig::new().with_params(params_obj.keys().map(|s| s.as_str()))
            }
            _ => {
                // No params provided - validation enabled with empty set
                ParserConfig::new()
            }
        };

        // Output format: ID, then either AST dump or error
        writeln!(stdout, "=== {} ===", id).unwrap();
        writeln!(stdout, "query: {:?}", query.trim()).unwrap();

        let mut parser = Parser::new_with_config(query, config);
        match parser.parse() {
            Ok(result) => {
                writeln!(stdout, "status: ok").unwrap();
                writeln!(stdout, "ast:").unwrap();
                let dump = dump_expr(&result.expr);
                for line in dump.lines() {
                    writeln!(stdout, "  {}", line).unwrap();
                }
            }
            Err(e) => {
                writeln!(stdout, "status: error").unwrap();
                writeln!(stdout, "error: {}", e).unwrap();
            }
        }
        writeln!(stdout).unwrap();
    }
}
