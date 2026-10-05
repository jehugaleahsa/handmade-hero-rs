# We build once with no features and once with all features.
# This realized feature-only dead code and unused parameter warnings.
&cargo fmt --check `
    && cargo clippy --all-features -- -Dwarnings `
    && cargo build `
    && cargo test --all-features
