import "infra/just/shells.just"
import "infra/rust/just/clippy.just"
import "infra/rust/just/format.just"

default:
    @just -l

[doc('Build the project')]
[group('dev')]
build:
    cargo build --all -r

[doc('Run tests')]
[group('dev')]
test:
    cargo test --all
