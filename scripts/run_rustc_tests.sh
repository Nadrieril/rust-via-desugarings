#!/usr/bin/env bash

# Run rustc test suites using our test driver using nightly.
# This script leverages the rustc's repo compiletest crate.
#
# The suites configuration should match:
# https://github.com/rust-lang/rust/blob/main/src/bootstrap/src/core/build_steps/test.rs
#
# Copied from https://github.com/rust-lang/project-stable-mir/blob/dd70d312ecbf4de0b74cba1c91aa7bfc5715a91f/.github/scripts/run_rustc_tests.sh

set -e
set -u
export RUST_BACKTRACE=1

# Location of a rust repository. Clone one if path doesn't exist.
RUST_REPO="${RUST_REPO:-"/tmp/rustc-for-tests"}"

DRIVER_PATH="$PWD/target/debug/rust-via-desugarings"

# Set up rustc repository
function setup_rustc_repo() {
  if [[ ! -e "${RUST_REPO}" ]]; then
    mkdir -p "$(dirname ${RUST_REPO})"
    git clone -b main https://github.com/rust-lang/rust.git "${RUST_REPO}"
    pushd "${RUST_REPO}"
    commit="$(rustc -vV | awk '/^commit-hash/ { print $2 }')"
    git checkout ${commit}
    git submodule init -- "${RUST_REPO}/library/stdarch"
    git submodule update
  else
    pushd "${RUST_REPO}"
  fi
}

function run_tests() {
  # Run the following suite configuration for now (test suite + mode)
  SUITES=(
    # "codegen codegen"
    # "codegen-units codegen-units"
    "ui ui"
    #"mir-opt mir-opt"
    #"pretty pretty"
  )

  SYSROOT=$(rustc --print sysroot)
  PY_PATH=$(type -P python3)
  HOST=$(rustc -vV | awk '/^host/ { print $2 }')
  # FILE_CHECK="$(which FileCheck-12 || which FileCheck-13 || which FileCheck-14)"

  for suite_cfg in "${SUITES[@]}"; do
    # Hack to work on older bash like the ones on MacOS.
    suite_pair=($suite_cfg)
    suite=${suite_pair[0]}
    mode=${suite_pair[1]}

    echo "#### Running suite: ${suite} mode: ${mode}"
    cargo run -p compiletest -- \
      --compile-lib-path="${SYSROOT}/lib" \
      --run-lib-path="${SYSROOT}/lib"\
      --python="${PY_PATH}" \
      --rustc-path="${DRIVER_PATH}" \
      --mode=${mode} \
      --suite="${suite}" \
      --src-root="$PWD" \
      --src-test-suite-root="$PWD/tests/${suite}" \
      --minicore-path="$PWD/tests/auxiliary/minicore" \
      --build-root="$PWD/build" \
      --build-test-suite-root="$PWD/build/${HOST}/tests/${suite}" \
      --sysroot-base="$SYSROOT" \
      --stage=1 \
      --stage-id=stage1-${HOST} \
      --cc= \
      --cxx= \
      --cflags= \
      --cxxflags= \
      --llvm-components= \
      --android-cross-path= \
      --channel=nightly \
      --nightly-branch=whatever \
      --git-merge-commit-email=whatever \
      --target=${HOST}
      # --llvm-filecheck="${FILE_CHECK}" \
  done
}

setup_rustc_repo
run_tests
