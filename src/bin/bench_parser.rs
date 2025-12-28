use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

use groq_parser::parser::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: bench_parser <test-suite.ndjson> [seconds]");
        std::process::exit(1);
    }

    let duration_secs: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let duration = Duration::from_secs(duration_secs);

    let file = File::open(&args[1]).expect("Failed to open file");
    let reader = BufReader::new(file);

    // First pass: extract all queries
    let mut queries: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line)
            && let Some(query) = json.get("query").and_then(|q| q.as_str())
        {
            queries.push(query.to_string());
        }
    }

    eprintln!(
        "Loaded {} queries, running for {} seconds...",
        queries.len(),
        duration_secs
    );

    // Warm up
    for query in queries.iter().take(100) {
        let mut parser = Parser::new(query);
        let _ = parser.parse();
    }

    // Benchmark: run for N seconds
    let mut iterations = 0u64;
    let mut success = 0u64;
    let mut failure = 0u64;

    let start = Instant::now();
    while start.elapsed() < duration {
        for query in &queries {
            let mut parser = Parser::new(query);
            match parser.parse() {
                Ok(_) => success += 1,
                Err(_) => failure += 1,
            }
        }
        iterations += 1;
    }
    let elapsed = start.elapsed();

    let total_queries = iterations * queries.len() as u64;
    let per_query = elapsed.as_nanos() as f64 / total_queries as f64;

    println!("Rust Parser Results:");
    println!("  Iterations: {}", iterations);
    println!(
        "  Queries:    {} ({} per iteration)",
        total_queries,
        queries.len()
    );
    println!("  Success:    {}", success);
    println!("  Failure:    {}", failure);
    println!("  Time:       {:?}", elapsed);
    println!(
        "  Per query:  {:.0}ns ({:.2}µs)",
        per_query,
        per_query / 1000.0
    );
}
