package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	groq "github.com/sanity-io/go-groq"
	"github.com/sanity-io/go-groq/ast"
	"github.com/sanity-io/go-groq/parser"
)

type TestEntry struct {
	Type   string                 `json:"_type"`
	ID     string                 `json:"_id"`
	Query  *string                `json:"query"`
	Valid  *bool                  `json:"valid"`
	Params map[string]interface{} `json:"params"`
}

func main() {
	scanner := bufio.NewScanner(os.Stdin)
	// Increase buffer size for long lines
	buf := make([]byte, 0, 64*1024)
	scanner.Buffer(buf, 1024*1024)

	for scanner.Scan() {
		line := scanner.Text()
		if strings.TrimSpace(line) == "" {
			continue
		}

		var entry TestEntry
		if err := json.Unmarshal([]byte(line), &entry); err != nil {
			continue
		}

		// Skip dataset entries
		if entry.Type != "test" {
			continue
		}

		if entry.Query == nil {
			continue
		}

		query := *entry.Query
		id := entry.ID
		if id == "" {
			id = "unknown"
		}

		fmt.Printf("=== %s ===\n", id)
		fmt.Printf("query: %q\n", strings.TrimSpace(query))

		// Build parser options with params from test entry
		var opts []parser.Option
		if entry.Params != nil {
			// Pass the params map directly - Go uses it for validation
			opts = append(opts, parser.WithParams(groq.Params(entry.Params)))
		}

		expr, err := parser.Parse(query, opts...)
		if err != nil {
			fmt.Printf("status: error\n")
			fmt.Printf("error: %s\n", err.Error())
			fmt.Println()
			continue
		}

		fmt.Printf("status: ok\n")
		fmt.Printf("ast:\n")
		dumpExpr(expr, 1)
		fmt.Println()
	}
}

func indent(level int) {
	for i := 0; i < level; i++ {
		fmt.Print("  ")
	}
}

func tokenSymbol(tok ast.Token) string {
	switch tok {
	case ast.Equals:
		return "=="
	case ast.NEQ:
		return "!="
	case ast.GT:
		return ">"
	case ast.LT:
		return "<"
	case ast.GTE:
		return ">="
	case ast.LTE:
		return "<="
	case ast.And:
		return "&&"
	case ast.Or:
		return "||"
	case ast.Not:
		return "!"
	case ast.Plus:
		return "+"
	case ast.Minus:
		return "-"
	case ast.Asterisk:
		return "*"
	case ast.Slash:
		return "/"
	case ast.Percent:
		return "%"
	case ast.Exponentiation:
		return "**"
	case ast.Pipe:
		return "|"
	case ast.Arrow:
		return "->"
	case ast.Dot:
		return "."
	case ast.DotDot:
		return ".."
	case ast.DotDotDot:
		return "..."
	case ast.Colon:
		return ":"
	case ast.MatchOperator:
		return "match"
	case ast.InOperator:
		return "in"
	case ast.AscOperator:
		return "asc"
	case ast.DescOperator:
		return "desc"
	case ast.Rocket:
		return "=>"
	default:
		return "<??>"
	}
}

func dumpExpr(expr ast.Expression, level int) {
	switch e := expr.(type) {
	case *ast.Everything:
		indent(level)
		fmt.Println("Everything")

	case *ast.This:
		indent(level)
		fmt.Println("This")

	case *ast.Parent:
		indent(level)
		fmt.Println("Parent")

	case *ast.Constraint:
		indent(level)
		fmt.Println("Constraint")
		indent(level + 1)
		fmt.Println("expression:")
		dumpExpr(e.Expression, level+2)

	case *ast.Object:
		indent(level)
		fmt.Println("Object")
		for i, item := range e.Expressions {
			indent(level + 1)
			fmt.Printf("[%d]:\n", i)
			dumpExpr(item, level+2)
		}

	case *ast.Array:
		indent(level)
		fmt.Println("Array")
		for i, item := range e.Expressions {
			indent(level + 1)
			fmt.Printf("[%d]:\n", i)
			dumpExpr(item, level+2)
		}

	case *ast.Subscript:
		indent(level)
		fmt.Println("Subscript")
		indent(level + 1)
		fmt.Println("value:")
		dumpExpr(e.Value, level+2)

	case *ast.Range:
		indent(level)
		op := "..."
		if e.Inclusive {
			op = ".."
		}
		fmt.Printf("Range %s\n", op)
		indent(level + 1)
		fmt.Println("start:")
		dumpExpr(e.Start, level+2)
		indent(level + 1)
		fmt.Println("end:")
		dumpExpr(e.End, level+2)

	case *ast.FunctionCall:
		indent(level)
		if e.Namespace == "" {
			fmt.Printf("FunctionCall %s\n", e.Name)
		} else {
			fmt.Printf("FunctionCall %s::%s\n", e.Namespace, e.Name)
		}
		for i, arg := range e.Arguments {
			indent(level + 1)
			fmt.Printf("arg[%d]:\n", i)
			dumpExpr(arg, level+2)
		}

	case *ast.BinaryOperator:
		indent(level)
		fmt.Printf("Binary %s\n", tokenSymbol(e.Operator))
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("rhs:")
		dumpExpr(e.RHS, level+2)

	case *ast.DotOperator:
		indent(level)
		fmt.Println("Dot")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("rhs:")
		dumpExpr(e.RHS, level+2)

	case *ast.ArrayTraversal:
		indent(level)
		fmt.Println("ArrayTraversal")
		indent(level + 1)
		fmt.Println("expr:")
		if e.Expr != nil {
			dumpExpr(e.Expr, level+2)
		} else {
			indent(level + 2)
			fmt.Println("Everything")
		}

	case *ast.Group:
		indent(level)
		fmt.Println("Group")
		indent(level + 1)
		fmt.Println("expression:")
		dumpExpr(e.Expression, level+2)

	case *ast.Tuple:
		indent(level)
		fmt.Println("Tuple")
		for i, m := range e.Members {
			indent(level + 1)
			fmt.Printf("[%d]:\n", i)
			dumpExpr(m, level+2)
		}

	case *ast.PipeOperator:
		indent(level)
		fmt.Println("Pipe")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("rhs:")
		dumpExpr(e.RHS, level+2)

	case *ast.PrefixOperator:
		indent(level)
		fmt.Printf("Prefix %s\n", tokenSymbol(e.Operator))
		indent(level + 1)
		fmt.Println("rhs:")
		dumpExpr(e.RHS, level+2)

	case *ast.PostfixOperator:
		indent(level)
		fmt.Printf("Postfix %s\n", tokenSymbol(e.Operator))
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)

	case *ast.Attribute:
		indent(level)
		fmt.Printf("Attribute %q\n", e.Name)

	case *ast.Param:
		indent(level)
		fmt.Printf("Param %q\n", e.Name)

	case *ast.StringLiteral:
		indent(level)
		fmt.Printf("String %q\n", e.Value)

	case *ast.IntegerLiteral:
		indent(level)
		fmt.Printf("Integer %d\n", e.Value)

	case *ast.FloatLiteral:
		indent(level)
		fmt.Printf("Float %v\n", e.Value)

	case *ast.BooleanLiteral:
		indent(level)
		fmt.Printf("Boolean %v\n", e.Value)

	case *ast.NullLiteral:
		indent(level)
		fmt.Println("Null")

	case *ast.Projection:
		indent(level)
		fmt.Println("Projection")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("object:")
		dumpExpr(e.Object, level+2)

	case *ast.FunctionPipe:
		indent(level)
		fmt.Println("FunctionPipe")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("func:")
		dumpExpr(e.Func, level+2)

	case *ast.Filter:
		indent(level)
		fmt.Println("Filter")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("constraint:")
		dumpExpr(e.Constraint, level+2)

	case *ast.Element:
		indent(level)
		fmt.Println("Element")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("idx:")
		dumpExpr(e.Idx, level+2)

	case *ast.Slice:
		indent(level)
		fmt.Println("Slice")
		indent(level + 1)
		fmt.Println("lhs:")
		dumpExpr(e.LHS, level+2)
		indent(level + 1)
		fmt.Println("range:")
		dumpExpr(e.Range, level+2)

	case *ast.Ellipsis:
		indent(level)
		fmt.Println("Ellipsis")

	default:
		indent(level)
		fmt.Printf("Unknown<%T>\n", expr)
	}
}
