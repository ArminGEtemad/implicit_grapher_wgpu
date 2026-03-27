# Transpiler

This is a bridge between human readable math and shader code. It has three stages:

- Lexer
- Abstract Syntax Tree (AST)
- Parser

## Lexer

The Lexer gets the raw string input and groups it into meaningful tokens. This tokens are either

- literals
- Operationals
- Functionals
- controls

Literals are numbers and variables and operationals are such as addition, multiplication etc. Functionals are functions such as sine and cosine. The parentheses are for control. A user input such as `3.0 + x^2 - sin(y)` would become `[Number(3.0), Add, Variable(x), Exp, 2.0, Sub, Sin, Variable(y)]`.

## AST

Before generating the code, we have to make sure that the order of operation is preserved. If it sees an operation, it uses `Binary` expression where we have

- (
- left expression
- operation
- right expression
- )

In the case of `Unary` it is a function like

- function
- (
- expression
- )

## Parser

Parser gets the list of tokens and builds the expression tree.

| Level | Function         | Handles                | Precedence |
| ----- | ---------------- | ---------------------- | ---------- |
| 1     | weak_handle      | +, -                   | Lowest     |
| 2     | strong_handle    | \*, /                  | Medium     |
| 3     | power_handle     | ^ (Exp)                | High       |
| 4     | primary_handling | (), Functions, Numbers | Highest    |

The `to_wgsl_code` method performs the final transformation.
Something like `3.0 + x^2 - sin(y)` would become `(3.00 + pow(p.x, 2.00) - sin(p.y))`
