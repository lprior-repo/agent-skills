#!/usr/bin/env bash
set -euo pipefail

# Rust Verification Gauntlet
#
# Usage:
#   ./rust-verification-gauntlet.sh fast
#   ./rust-verification-gauntlet.sh standard
#   ./rust-verification-gauntlet.sh deep
#   ./rust-verification-gauntlet.sh proof
#   ./rust-verification-gauntlet.sh all
#
# Philosophy:
#   fast: deterministic development gate
#   standard: normal pre-merge gate
#   deep: high-assurance local gate
#   proof: formal/bounded proof gate
#   all: everything in sequence

MODE="${1:-standard}"

CHECK_FLAGS="${CHECK_FLAGS:---workspace --all-targets --all-features --locked}"
SOURCE_CLIPPY_FLAGS="${SOURCE_CLIPPY_FLAGS:---workspace --lib --bins --examples --all-features --locked}"
TEST_FLAGS="${TEST_FLAGS:---workspace --all-features --locked}"
MIRI_FLAGS="${MIRI_FLAGS:---workspace --all-targets}"
VERUS_CMD="${VERUS_CMD:-}"
TLA_CMD="${TLA_CMD:-}"
KANI_FLAGS="${KANI_FLAGS:---workspace}"
BOLERO_ENGINE="${BOLERO_ENGINE:-libfuzzer}"
BOLERO_TIME="${BOLERO_TIME:-60s}"
MUTANTS_FLAGS="${MUTANTS_FLAGS:---workspace}"
LOCKBUD_FLAGS="${LOCKBUD_FLAGS:--k all .}"

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

section() {
  printf '\n== %s ==\n' "$1"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    printf 'install it before running this gate\n' >&2
    exit 127
  fi
}

optional_cmd() {
  command -v "$1" >/dev/null 2>&1
}

gate_fast() {
  section "deterministic compile, lint, and tests"
  # `check` covers tests/examples/benches. Strict clippy is source-only so test
  # implementation style never becomes a rejection gate.
  cargo fmt --all --check
  cargo check $CHECK_FLAGS
  cargo clippy $SOURCE_CLIPPY_FLAGS -- -D warnings
  cargo test $TEST_FLAGS --no-run

  if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run $TEST_FLAGS
  else
    cargo test $TEST_FLAGS
  fi
}

gate_policy() {
  section "dependency and unsafe policy"
  require_cmd cargo-deny
  cargo deny check

  if optional_cmd cargo-geiger; then
    cargo geiger --all-features
  else
    printf 'cargo-geiger not installed; optional unless selected by proof-obligations.planned.jsonl\n'
  fi
}

gate_ub() {
  section "undefined behavior and extra runtime checks"
  rustup component add miri --toolchain nightly >/dev/null
  cargo +nightly miri setup
  cargo +nightly miri test $MIRI_FLAGS

  if optional_cmd cargo-careful; then
    cargo +nightly careful test --workspace --all-targets
  else
    printf 'cargo-careful not installed; optional unless selected by proof-obligations.planned.jsonl\n'
  fi
}

gate_bolero() {
  section "Bolero property, fuzz, and Kani-compatible harnesses"
  if optional_cmd cargo-bolero; then
    cargo bolero test
    cargo bolero test --engine "$BOLERO_ENGINE" --time "$BOLERO_TIME"
  else
    printf 'cargo-bolero not installed; optional unless selected by proof-obligations.planned.jsonl\n'
  fi
}

gate_kani() {
  section "Kani bounded model checking"
  require_cmd cargo-kani
  cargo kani $KANI_FLAGS

  if optional_cmd cargo-bolero; then
    cargo bolero test --engine kani
  fi
}

gate_verus() {
  section "Verus Rust-native proof obligations"
  if [[ -n "$VERUS_CMD" ]]; then
    bash -lc "$VERUS_CMD"
    return
  fi

  if [[ -x ./scripts/verify-verus.sh ]]; then
    ./scripts/verify-verus.sh
    return
  fi

  if [[ "${VERUS_REQUIRED:-0}" = "1" ]]; then
    printf 'Verus is required by proof-obligations.planned.jsonl but no VERUS_CMD or executable ./scripts/verify-verus.sh is configured.\n' >&2
    exit 1
  fi

  printf 'Verus command not configured; optional unless selected by proof-obligations.planned.jsonl\n'
}

gate_tla() {
  section "TLA+ temporal model checking"
  if [[ -n "$TLA_CMD" ]]; then
    bash -lc "$TLA_CMD"
    return
  fi

  if [[ -x ./scripts/verify-tla.sh ]]; then
    ./scripts/verify-tla.sh
    return
  fi

  if [[ "${TLA_REQUIRED:-0}" = "1" ]]; then
    printf 'TLA+ is required by proof-obligations.planned.jsonl but no TLA_CMD or executable ./scripts/verify-tla.sh is configured.\n' >&2
    exit 1
  fi

  printf 'TLA+ command not configured; optional unless selected by proof-obligations.planned.jsonl\n'
}

gate_concurrency() {
  section "concurrency schedule and deadlock checks"
  RUSTFLAGS="--cfg loom" cargo test --workspace loom

  if optional_cmd lockbud; then
    lockbud $LOCKBUD_FLAGS
  else
    printf 'lockbud not installed; optional unless selected by proof-obligations.planned.jsonl\n'
  fi
}

gate_mutation() {
  section "mutation testing"
  require_cmd cargo-mutants
  cargo mutants $MUTANTS_FLAGS
}

gate_lean() {
  section "Lean formal proof kernel"
  if [[ -x ./scripts/verify-lean.sh ]]; then
    ./scripts/verify-lean.sh
  else
    printf 'missing executable ./scripts/verify-lean.sh\n' >&2
    printf 'create this script for your Aeneas/Charon/Lean proof crate\n' >&2
    exit 1
  fi
}

case "$MODE" in
  fast)
    gate_fast
    ;;
  standard)
    gate_fast
    gate_policy
    gate_ub
    gate_bolero
    gate_kani
    ;;
  deep)
    gate_fast
    gate_policy
    gate_ub
    gate_bolero
    gate_kani
    gate_concurrency
    gate_mutation
    ;;
  proof)
    gate_tla
    gate_verus
    gate_kani
    gate_lean
    ;;
  all)
    gate_fast
    gate_policy
    gate_ub
    gate_bolero
    gate_tla
    gate_verus
    gate_kani
    gate_concurrency
    gate_mutation
    gate_lean
    ;;
  *)
    printf 'unknown mode: %s\n' "$MODE" >&2
    exit 2
    ;;
esac
