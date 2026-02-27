#!/usr/bin/env bash
# Full validation script: build + test + lint + format check
set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

pass() { echo -e "${GREEN}${BOLD}PASS${NC} $1"; }
fail() { echo -e "${RED}${BOLD}FAIL${NC} $1"; exit 1; }

echo -e "${BOLD}=== CCC Development Validation ===${NC}"
echo ""

echo -e "${BOLD}[1/4] Building...${NC}"
cargo build --release 2>&1 && pass "cargo build --release" || fail "cargo build --release"

echo ""
echo -e "${BOLD}[2/4] Running tests...${NC}"
cargo test --release 2>&1 && pass "cargo test --release" || fail "cargo test --release"

echo ""
echo -e "${BOLD}[3/4] Running clippy...${NC}"
cargo clippy -- -D warnings 2>&1 && pass "cargo clippy" || fail "cargo clippy"

echo ""
echo -e "${BOLD}[4/4] Checking formatting...${NC}"
cargo fmt --check 2>&1 && pass "cargo fmt --check" || fail "cargo fmt --check"

echo ""
echo -e "${GREEN}${BOLD}All checks passed!${NC}"
