[doc('Run clippy')]
[group('dev')]
lint:
    cargo clippy --all --tests -- -D warnings
