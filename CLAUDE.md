# CLAUDE.md — Development Guide

## Quick Reference

```bash
# Build (release mode required for all targets)
cargo build --release

# Run all unit tests (~512 tests)
cargo test --release

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

## Testing

### Unit Tests

All unit tests are in-source `#[test]` functions. Always run in release mode:

```bash
cargo test --release
```

### Manual End-to-End Testing

```bash
# Write a test program
echo 'int main() { return 42; }' > /tmp/test.c

# Compile with CCC
./target/release/ccc -o /tmp/test /tmp/test.c

# Run and check exit code
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
