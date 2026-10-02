# just list
default:
    @just --list --unsorted

# Everything CI checks: formatting, clippy and unit tests
[group('ci')]
check: fmt-check lint test

# Check formatting, without changing files
[group('ci')]
fmt-check:
    cargo fmt -- --check

# Clippy as in CI, with warnings as errors
[group('ci')]
lint:
    cargo clippy --all --all-features --tests --locked -- -D warnings

# Unit and doc tests (all but e2e), f. ex. `just test select`
[group('ci')]
[positional-arguments]
test *args:
    cargo test --all --all-features --locked --exclude vertigo-forms-e2e -- "$@"

# Storybook tested in Chrome, f. ex. `just e2e resource_table` or `just e2e --include-ignored`
[group('e2e')]
[positional-arguments]
e2e *args:
    cargo test -p vertigo-forms-e2e -- "$@"

# E2E tests in a visible browser, best for a few, f. ex. `just e2e-headed rejected_save`
[group('e2e')]
[positional-arguments]
e2e-headed *args:
    E2E_HEADLESS=0 cargo test -p vertigo-forms-e2e -- "$@"

# CI checks and e2e tests
all: check e2e
