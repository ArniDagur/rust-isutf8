#!/usr/bin/env bash

set -ex

: ${TARGET?"The TARGET environment variable must be set."}

export CARGO_SUBCMD=test
if [[ "${NORUN}" == "1" ]]; then
    export CARGO_SUBCMD=build
fi

# The source directory is read-only. Need to copy internal crates to the target
# directory for their Cargo.lock to be properly written.
mkdir target || true

rustc --version
cargo --version
echo "TARGET=${TARGET}"
echo "HOST=${HOST}"
echo "RUSTFLAGS=${RUSTFLAGS}"
echo "NORUN=${NORUN}"
echo "NOVERIFY=${NOVERIFY}"
echo "CARGO_SUBCMD=${CARGO_SUBCMD}"
echo "CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}"
echo "CARGO_INCREMENTAL=${CARGO_INCREMENTAL}"
echo "RUST_TEST_THREADS=${RUST_TEST_THREADS}"
echo "RUST_BACKTRACE=${RUST_BACKTRACE}"
echo "RUST_TEST_NOCAPTURE=${RUST_TEST_NOCAPTURE}"

cargo_test() {
    cmd="cargo ${CARGO_SUBCMD} --verbose --target=${TARGET} ${@}"
    mkdir target || true
    ${cmd} 2>&1 | tee > target/output
    if [[ ${PIPESTATUS[0]} != 0 ]]; then
        cat target/output
        return 1
    fi
}

# Run in debug mode
cargo_test

# Run in release mode
cargo_test --release
