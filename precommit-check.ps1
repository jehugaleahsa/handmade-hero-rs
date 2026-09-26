&cargo fmt --check `
    && cargo clippy -- -Dwarnings `
    && cargo build `
    && cargo test --all-features
