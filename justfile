default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-no-default-features:
    cargo clippy --workspace --no-default-features -- -D warnings

lint-alloc-feature:
    cargo clippy --workspace --no-default-features --features vouched/alloc,vouched-core/alloc -- -D warnings

lint-valuable-feature:
    cargo clippy --workspace --no-default-features --features vouched/valuable,vouched-core/valuable -- -D warnings

test:
    cargo test --workspace

test-all-features:
    cargo test --workspace --all-features

test-valuable-feature:
    cargo test -p vouched --features valuable

test-examples:
    cargo run -p vouched --example string_constraints
    cargo run -p vouched --example numeric_range_and_impls
    cargo run -p vouched --example error

check:
    cargo check --workspace --all-targets --all-features

check-no-default-features:
    cargo check --workspace --no-default-features

check-alloc-feature:
    cargo check --workspace --no-default-features --features vouched/alloc,vouched-core/alloc

check-valuable-feature:
    cargo check --workspace --no-default-features --features vouched/valuable,vouched-core/valuable

doc-test:
    cargo test --doc --workspace --all-features

doc:
    cargo doc --workspace --all-features --no-deps
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

deny:
    cargo deny check --hide-inclusion-graph

ci: fmt-check lint lint-no-default-features lint-alloc-feature lint-valuable-feature check check-no-default-features check-alloc-feature check-valuable-feature test test-all-features test-valuable-feature test-examples doc-test doc deny
