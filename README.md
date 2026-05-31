# lox-rs

A bytecode virtual machine for the [Lox](https://craftinginterpreters.com) programming language, written in Rust.

## Usage

```sh
cargo run
```

Starts a REPL. Enter Lox expressions at the `>` prompt.

```sh
cargo run -- script.lox
```

Runs a file.

## Features

- Pratt parser for expression precedence
- Bytecode compiler emitting stack-based instructions
- Stack-based VM with constant pool
- REPL and file execution

## Debug

```sh
cargo run --features debug_trace_execution
```

Prints instruction trace during execution (enabled by default).

## Status

Early stages — numeric expressions with basic arithmetic (`+`, `-`, `*`, `/`, unary `-`, grouping with `()`).
