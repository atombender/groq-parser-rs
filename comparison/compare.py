#!/usr/bin/env python3
"""
Compare AST dumps between Rust and Go GROQ parsers.

Usage:
    python compare.py <test-suite.ndjson> <rust-binary> <go-binary> [--version VERSION]

Example:
    python compare.py ../groq-test-suite.ndjson ../target/debug/dump_ast ./go-dumper/go-dumper --version 1.3
"""

import subprocess
import sys
import re
import json
from collections import defaultdict

# Target GROQ spec version
TARGET_VERSION = (1, 3)


def parse_version(version_str: str) -> tuple:
    """Parse version string like '1.0', '1.2', etc. to tuple (major, minor)"""
    if not version_str:
        return (1, 0)  # Default
    parts = version_str.strip().split('.')
    try:
        return (int(parts[0]), int(parts[1]) if len(parts) > 1 else 0)
    except (ValueError, IndexError):
        return (1, 0)


def version_matches(version_spec: str, target: tuple) -> bool:
    """
    Check if a version spec matches the target version.

    Specs:
    - "0.1" -> exact match, only version 0.1
    - ">= 1.0" -> version 1.0 and later
    - ">= 1.1" -> version 1.1 and later
    """
    if not version_spec:
        return True  # No version specified, include by default

    version_spec = version_spec.strip()

    if version_spec.startswith('>= '):
        min_version = parse_version(version_spec[3:])
        return target >= min_version
    elif version_spec.startswith('> '):
        min_version = parse_version(version_spec[2:])
        return target > min_version
    elif version_spec.startswith('<= '):
        max_version = parse_version(version_spec[3:])
        return target <= max_version
    elif version_spec.startswith('< '):
        max_version = parse_version(version_spec[2:])
        return target < max_version
    else:
        # Exact version match
        exact_version = parse_version(version_spec)
        return target == exact_version


def filter_test_file(test_file: str, target_version: tuple) -> tuple:
    """
    Filter test entries by version and return (filtered_content, stats).
    Returns the filtered NDJSON content as a string.
    """
    filtered_lines = []
    stats = {'total': 0, 'included': 0, 'excluded': 0, 'excluded_versions': defaultdict(int)}

    with open(test_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue

            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue

            # Only process test entries
            if entry.get('_type') != 'test':
                filtered_lines.append(line)
                continue

            stats['total'] += 1
            version_spec = entry.get('version', '')

            if version_matches(version_spec, target_version):
                filtered_lines.append(line)
                stats['included'] += 1
            else:
                stats['excluded'] += 1
                stats['excluded_versions'][version_spec] += 1

    return '\n'.join(filtered_lines), stats


def parse_dump_output(output: str) -> dict:
    """Parse the dump output into a dict of {test_id: (query, status, ast_or_error)}"""
    results = {}
    current_id = None
    current_query = None
    current_status = None
    current_ast = []

    lines = output.split('\n')
    i = 0
    while i < len(lines):
        line = lines[i]

        # Match test header: === test-id ===
        match = re.match(r'^=== (.+) ===$', line)
        if match:
            # Save previous entry
            if current_id is not None:
                ast_str = '\n'.join(current_ast).rstrip()
                results[current_id] = (current_query, current_status, ast_str)

            current_id = match.group(1)
            current_query = None
            current_status = None
            current_ast = []
            i += 1
            continue

        # Match query line
        if line.startswith('query: '):
            current_query = line[7:]
            i += 1
            continue

        # Match status line
        if line.startswith('status: '):
            current_status = line[8:]
            i += 1
            continue

        # Match ast: line
        if line == 'ast:':
            i += 1
            # Collect AST lines until next test or end
            while i < len(lines) and not lines[i].startswith('===') and lines[i] != '':
                current_ast.append(lines[i])
                i += 1
            continue

        # Match error line
        if line.startswith('error: '):
            current_ast = [line[7:]]
            i += 1
            continue

        i += 1

    # Save last entry
    if current_id is not None:
        ast_str = '\n'.join(current_ast).rstrip()
        results[current_id] = (current_query, current_status, ast_str)

    return results


def run_dumper(binary: str, input_content: str) -> str:
    """Run a dumper binary with the given input and return its output"""
    result = subprocess.run(
        [binary],
        input=input_content,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    return result.stdout


def categorize_diff(rust_line: str, go_line: str) -> str:
    """Categorize a line difference"""
    # Float formatting (scientific notation)
    if 'Float ' in rust_line and 'Float ' in go_line:
        rust_val = rust_line.split('Float ')[1].strip()
        go_val = go_line.split('Float ')[1].strip()
        # Check for scientific notation vs expanded
        if ('e' in go_val.lower() or 'e' in rust_val.lower()) and \
           ('e' not in go_val.lower() or 'e' not in rust_val.lower()):
            return 'float_sci_notation'
        # Check for tiny precision differences
        try:
            r = float(rust_val)
            g = float(go_val)
            if abs(r - g) / max(abs(r), abs(g), 1) < 1e-14:
                return 'float_precision'
        except:
            pass
        return 'float_other'

    # Integer vs Float (large number handling)
    if ('Integer ' in rust_line and 'Float ' in go_line) or \
       ('Float ' in rust_line and 'Integer ' in go_line):
        return 'int_overflow'

    # Array traversal differences
    if 'ArrayTraversal' in rust_line or 'ArrayTraversal' in go_line:
        return 'array_traversal'

    return 'other'


def compare_asts(rust_ast: str, go_ast: str) -> tuple:
    """Compare two AST strings line by line, return differences and categories"""
    rust_lines = rust_ast.split('\n')
    go_lines = go_ast.split('\n')

    diffs = []
    categories = defaultdict(int)
    max_lines = max(len(rust_lines), len(go_lines))

    for i in range(max_lines):
        rust_line = rust_lines[i] if i < len(rust_lines) else '<missing>'
        go_line = go_lines[i] if i < len(go_lines) else '<missing>'

        if rust_line != go_line:
            cat = categorize_diff(rust_line, go_line)
            categories[cat] += 1
            diffs.append((i + 1, rust_line, go_line, cat))

    return diffs, dict(categories)


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        sys.exit(1)

    test_file = sys.argv[1]
    rust_binary = sys.argv[2]
    go_binary = sys.argv[3]

    # Parse optional --version argument
    target_version = TARGET_VERSION
    if '--version' in sys.argv:
        idx = sys.argv.index('--version')
        if idx + 1 < len(sys.argv):
            target_version = parse_version(sys.argv[idx + 1])

    print(f"Target GROQ version: {target_version[0]}.{target_version[1]}")
    print(f"Filtering test suite: {test_file}")
    filtered_content, filter_stats = filter_test_file(test_file, target_version)
    print(f"  Total tests: {filter_stats['total']}")
    print(f"  Included:    {filter_stats['included']}")
    print(f"  Excluded:    {filter_stats['excluded']}")
    if filter_stats['excluded_versions']:
        for ver, count in sorted(filter_stats['excluded_versions'].items()):
            print(f"    - {ver!r}: {count}")

    print(f"\nRunning Rust dumper: {rust_binary}")
    rust_output = run_dumper(rust_binary, filtered_content)
    rust_results = parse_dump_output(rust_output)
    print(f"  Parsed {len(rust_results)} tests")

    print(f"Running Go dumper: {go_binary}")
    go_output = run_dumper(go_binary, filtered_content)
    go_results = parse_dump_output(go_output)
    print(f"  Parsed {len(go_results)} tests")

    # Compare results
    all_ids = set(rust_results.keys()) | set(go_results.keys())

    stats = {
        'total': len(all_ids),
        'match': 0,
        'status_mismatch': 0,
        'ast_mismatch': 0,
        'rust_only': 0,
        'go_only': 0,
    }

    category_counts = defaultdict(int)
    mismatches = []
    status_mismatches = []

    for test_id in sorted(all_ids):
        if test_id not in rust_results:
            stats['rust_only'] += 1
            mismatches.append((test_id, 'rust_missing', None, None))
            continue

        if test_id not in go_results:
            stats['go_only'] += 1
            mismatches.append((test_id, 'go_missing', None, None))
            continue

        rust_query, rust_status, rust_ast = rust_results[test_id]
        go_query, go_status, go_ast = go_results[test_id]

        if rust_status != go_status:
            stats['status_mismatch'] += 1
            status_mismatches.append((test_id, rust_query, rust_status, rust_ast, go_status, go_ast))
            continue

        if rust_status == 'error':
            # Both errored, consider it a match for now
            stats['match'] += 1
            continue

        if rust_ast != go_ast:
            stats['ast_mismatch'] += 1
            diffs, cats = compare_asts(rust_ast, go_ast)
            for cat, count in cats.items():
                category_counts[cat] += count
            mismatches.append((test_id, 'ast', rust_ast, go_ast, diffs, cats))
            continue

        stats['match'] += 1

    # Print summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Total tests:        {stats['total']}")
    print(f"Matching:           {stats['match']} ({100*stats['match']/stats['total']:.1f}%)")
    print(f"Status mismatch:    {stats['status_mismatch']}")
    print(f"AST mismatch:       {stats['ast_mismatch']}")
    print(f"Rust only:          {stats['rust_only']}")
    print(f"Go only:            {stats['go_only']}")

    if category_counts:
        print("\n" + "=" * 60)
        print("AST MISMATCH CATEGORIES (line diff counts)")
        print("=" * 60)
        for cat, count in sorted(category_counts.items(), key=lambda x: -x[1]):
            print(f"  {cat}: {count}")

    # Print status mismatches
    if status_mismatches:
        print("\n" + "=" * 60)
        print(f"STATUS MISMATCHES ({len(status_mismatches)} total, showing first 10)")
        print("=" * 60)

        for item in status_mismatches[:10]:
            test_id, query, rust_status, rust_ast, go_status, go_ast = item
            print(f"\n--- {test_id} ---")
            print(f"  Query: {query[:100]}..." if len(query) > 100 else f"  Query: {query}")
            print(f"  Rust status: {rust_status}")
            if rust_status == 'error':
                print(f"    Error: {rust_ast[:100]}..." if len(rust_ast) > 100 else f"    Error: {rust_ast}")
            print(f"  Go status:   {go_status}")
            if go_status == 'error':
                print(f"    Error: {go_ast[:100]}..." if len(go_ast) > 100 else f"    Error: {go_ast}")

    # Print float_sci_notation samples
    sci_mismatches = [m for m in mismatches if m[1] == 'ast' and
                        m[5].get('float_sci_notation', 0) > 0]
    if sci_mismatches:
        print("\n" + "=" * 60)
        print(f"FLOAT_SCI_NOTATION MISMATCHES ({len(sci_mismatches)} tests, showing first 5)")
        print("=" * 60)

        for item in sci_mismatches[:5]:
            test_id = item[0]
            diffs = item[4]
            rust_query = rust_results[test_id][0]
            print(f"\n--- {test_id} ---")
            print(f"  Query: {rust_query[:100]}..." if len(rust_query) > 100 else f"  Query: {rust_query}")
            sci_diffs = [d for d in diffs if d[3] == 'float_sci_notation']
            for line_no, rust_line, go_line, _ in sci_diffs[:2]:
                print(f"    Rust: {rust_line.strip()}")
                print(f"    Go:   {go_line.strip()}")

    # Print int_overflow samples
    int_mismatches = [m for m in mismatches if m[1] == 'ast' and
                        m[5].get('int_overflow', 0) > 0]
    if int_mismatches:
        print("\n" + "=" * 60)
        print(f"INT_OVERFLOW MISMATCHES ({len(int_mismatches)} tests, showing first 5)")
        print("=" * 60)

        for item in int_mismatches[:5]:
            test_id = item[0]
            diffs = item[4]
            rust_query = rust_results[test_id][0]
            print(f"\n--- {test_id} ---")
            print(f"  Query: {rust_query[:100]}..." if len(rust_query) > 100 else f"  Query: {rust_query}")
            int_diffs = [d for d in diffs if d[3] == 'int_overflow']
            for line_no, rust_line, go_line, _ in int_diffs[:2]:
                print(f"    Rust: {rust_line.strip()}")
                print(f"    Go:   {go_line.strip()}")

    # Print non-float AST mismatches
    other_mismatches = [m for m in mismatches if m[1] == 'ast' and
                        m[5].get('other', 0) > 0]
    if other_mismatches:
        print("\n" + "=" * 60)
        print(f"STRUCTURAL AST MISMATCHES (non-float, {len(other_mismatches)} total, showing first 10)")
        print("=" * 60)

        for item in other_mismatches[:10]:
            test_id = item[0]
            rust_ast = item[2]
            go_ast = item[3]
            diffs = item[4]
            cats = item[5]

            rust_query = rust_results[test_id][0]
            print(f"\n--- {test_id} ---")
            print(f"  Query: {rust_query[:100]}..." if len(rust_query) > 100 else f"  Query: {rust_query}")
            print(f"  Categories: {cats}")

            other_diffs = [d for d in diffs if d[3] == 'other']
            print(f"  'other' differences ({len(other_diffs)}):")
            for line_no, rust_line, go_line, _ in other_diffs[:3]:
                print(f"    Line {line_no}:")
                print(f"      Rust: {rust_line.strip()!r}")
                print(f"      Go:   {go_line.strip()!r}")

    # Exit with error code if there are mismatches
    if stats['match'] != stats['total']:
        sys.exit(1)


if __name__ == '__main__':
    main()
