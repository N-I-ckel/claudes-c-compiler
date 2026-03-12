# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Reference

```bash
# Build (release mode required for all targets)
cargo build --release

# Run all unit tests (~512 tests)
cargo test --release

# Run a single test by name pattern
cargo test --release test_name_pattern

# Lint (all warnings must pass)
cargo clippy -- -D warnings

# Format check
cargo fmt --check

# Full validation (build + test + lint + format)
bash scripts/test.sh
```

## Project Overview

CCC (Claude's C Compiler) is a full C compiler written in Rust with zero external dependencies. It targets x86-64, i686, AArch64, and RISC-V 64, producing ELF executables with a built-in assembler and linker.

## Build Targets

| Binary | Target | Usage |
|--------|--------|-------|
| `ccc` | x86-64 (default) | `./target/release/ccc -o out input.c` |
| `ccc-x86` | x86-64 | Same as `ccc` |
| `ccc-arm` | AArch64 | `./target/release/ccc-arm -o out input.c` |
| `ccc-riscv` | RISC-V 64 | `./target/release/ccc-riscv -o out input.c` |
| `ccc-i686` | i686 (32-bit x86) | `./target/release/ccc-i686 -o out input.c` |

### Optional Features

```bash
cargo build --release --features gcc_assembler,gcc_linker  # GCC fallback
cargo build --release --features gcc_m16                    # GCC for -m16 boot code
```

## Project Architecture

```
C Source → Preprocessor → Lexer → Parser → Sema → IR Lowering → mem2reg (SSA)
  → Optimization Passes (15 passes, up to 3 iterations) → Phi Elimination
  → Code Generation → Peephole Optimizer → Assembler → Linker → ELF
```

### Source Tree

```
src/
  frontend/     C source → typed AST (preprocessor, lexer, parser, sema)
  ir/           Target-independent SSA IR (lowering, mem2reg)
  passes/       SSA optimization passes (15 passes + shared loop analysis)
  backend/      IR → assembly → machine code → ELF (4 architectures)
    x86/        x86-64 code generation, assembler, linker
    i686/       i686 (32-bit x86) code generation, assembler, linker
    arm/        AArch64 code generation, assembler, linker
    riscv/      RISC-V 64 code generation, assembler, linker
  common/       Shared types, symbol table, diagnostics
  driver/       CLI parsing, pipeline orchestration
include/        Bundled C headers (SIMD intrinsics, NEON, etc.)
```

Each `src/` subdirectory has its own `README.md`. See also [DESIGN_DOC.md](DESIGN_DOC.md).

### Key Abstractions

- **`ArchCodegen` trait** (`src/backend/traits.rs`): ~185 methods that every backend implements. Shared default implementations call small "primitive" methods, so most backend work involves implementing or modifying these primitives. The `delegate_to_impl!` macro eliminates boilerplate.
- **IR types** (`src/ir/instruction.rs`): `Value(u32)` is an SSA virtual register, `BlockId(u32)` is a basic block label, `Operand` is either `Value` or `Const(IrConst)`, and `Instruction` is an enum with ~38 variants.
- **Dual type system** (`src/common/types.rs`): `CType` for C-level semantics (frontend), `IrType` for machine-level operations (14 variants: I8-I128, U8-U128, F32/F64/F128, Ptr, Void).

## Testing

All unit tests are in-source `#[test]` functions. Always run in release mode (debug is too slow):

```bash
cargo test --release                    # all tests
cargo test --release some_pattern       # filter by name
```

### Manual End-to-End Testing

```bash
echo 'int main() { return 42; }' > /tmp/test.c
./target/release/ccc -o /tmp/test /tmp/test.c
/tmp/test; echo $?  # Should print 42

# Cross-compile and test with QEMU
./target/release/ccc-arm -o /tmp/test-arm /tmp/test.c
qemu-aarch64 -L /usr/aarch64-linux-gnu /tmp/test-arm; echo $?

./target/release/ccc-riscv -o /tmp/test-riscv /tmp/test.c
qemu-riscv64 -L /usr/riscv64-linux-gnu /tmp/test-riscv; echo $?
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CCC_TIME_PHASES` | Print per-phase compilation timing to stderr |
| `CCC_TIME_PASSES` | Print per-pass optimization timing and change counts to stderr |
| `CCC_DISABLE_PASSES` | Disable specific optimization passes (comma-separated, or `all`) |
| `CCC_KEEP_ASM` | Preserve intermediate `.s` files next to output |
| `CCC_ASM_DEBUG` | Dump preprocessed assembly to `/tmp/asm_debug_<name>.s` |

## Task Tracking

Active bugs and tasks are tracked in text files:

- **`current_tasks/`** — Active bugs to fix. Each file describes a specific issue with repro steps.
- **`ideas/`** — Future improvement proposals and design ideas.

When starting work on a bug, read the corresponding file in `current_tasks/` for context and reproduction steps.

## Code Conventions

- **Rust 2021 edition**, zero external crates (only `std`)
- **Release mode only** — debug builds are too slow for testing compilation
- All warnings must be clean (`cargo clippy -- -D warnings`)
- Code formatting via `rustfmt` (`cargo fmt`)
- Trait-based backend abstraction — each architecture implements shared traits
- SSA-based IR with explicit phi nodes
- Peephole optimizers work on assembly text lines (string-based pattern matching)
