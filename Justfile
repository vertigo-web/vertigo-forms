set ignore-comments

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

# Storybook tested in Chrome, f. ex. `just e2e --headed resource_table` or `just e2e -- --include-ignored`
[group('e2e')]
[positional-arguments]
[arg('E2E_HEADLESS', long='headed', value='0', help="Show the browser window, best for a few tests (E2E_HEADLESS=0)")]
[arg('E2E_RELEASE', long='release', value='1', help="Build the storybook in release mode (E2E_RELEASE=1)")]
[arg('E2E_WASM_OPT', long='wasm-opt', value='1', help="Run wasm-opt on the storybook, as a shipped build has it (E2E_WASM_OPT=1)")]
[arg('E2E_SKIP_BUILD', long='skip-build', value='1', help="Use the last build (E2E_SKIP_BUILD=1)")]
e2e $E2E_HEADLESS=env('E2E_HEADLESS', '1') $E2E_RELEASE=env('E2E_RELEASE', '') $E2E_WASM_OPT=env('E2E_WASM_OPT', '') $E2E_SKIP_BUILD=env('E2E_SKIP_BUILD', '') *args:
    # The options are exported as the E2E_* variables the harness reads, an empty one counts
    # as off. They are also the first four positional arguments, so shifted out of "$@".
    shift 4; cargo test -p vertigo-forms-e2e -- "$@"

# CI checks and e2e tests
all: check e2e
