# Architecture — ternary-signals

> *Internal design and data flow.*

## Overview

This crate implements ternary {-1, 0, +1} semantics for the `signals` domain.
It is one of ~280 ternary crates in the SuperInstance fleet, all sharing Z₃ arithmetic
from [ternary-core](https://github.com/SuperInstance/ternary-core).

## Core Types

- **`FixedComplex`**

## Key Functions

- `from_i8()`
- `to_i8()`
- `new()`
- `zero()`
- `from_scaled()`
- `mul()`
- `add()`
- `mag_sq()`

## Ternary Mapping

| Value | Meaning |
|-------|---------|
| +1 | Excitatory |
| 0  | Neutral |
| -1 | Inhibitory |

## Source Structure

1 Rust source file(s) in `src/`.
Language: Rust

## Cross-Repo References

- [ternary-core](https://github.com/SuperInstance/ternary-core) — shared Z₃ traits
- [ternary-types](https://github.com/SuperInstance/ternary-types) — type-level encodings
- [Full SuperInstance fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)
