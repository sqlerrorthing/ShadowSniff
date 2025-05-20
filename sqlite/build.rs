fn main() {
    cc::Build::new()
        .file("sqlite3/sqlite3.c")
        .define("SQLITE_OMIT_LOAD_EXTENSION", None)
        .compile("sqlite3");

    println!("cargo:rustc-link-lib=static=sqlite3");
    println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
}