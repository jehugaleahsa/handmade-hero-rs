&cargo fmt --check `
    && cargo clippy -- -Dwarnings `
    && cargo test --all-features