release:
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none -Cdebuginfo=0 -Clink-args=-s" cargo build --release

debug:
    cargo build

run:
    cargo run