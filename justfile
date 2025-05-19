release:
    RUSTFLAGS="-Z location-detail=none \
               -Z fmt-debug=none \
               -C debuginfo=0 \
               -C link-arg=/OPT:REF \
               -C link-arg=/OPT:ICF \
               -C link-arg=/INCREMENTAL:NO \
               -C link-arg=/DEBUG:NONE \
               -C link-arg=/RELEASE \
               " \
    cargo build --release

debug:
    cargo build

run:
    cargo run