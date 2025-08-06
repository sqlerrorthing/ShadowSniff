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

#![feature(tuple_trait)]

use inquire::InquireError;
use proc_macro2::TokenStream;
use std::fs;
use std::io::Write;
use std::marker::Tuple;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub mod send_expr;
pub mod send_settings;
pub mod sender_service;
pub mod empty_log;

pub trait ToExpr<Args: Tuple = ()> {
    fn to_expr(&self, args: Args) -> TokenStream;
}

pub trait ToExprExt<Args: Tuple = ()>: ToExpr<Args> {
    fn to_expr_temp_file(&self, args: Args) -> PathBuf;
}

impl<T: ToExpr<Args>, Args: Tuple> ToExprExt<Args> for T {
    fn to_expr_temp_file(&self, args: Args) -> PathBuf {
        let mut expr_file: NamedTempFile = NamedTempFile::new().unwrap();
        expr_file.disable_cleanup(true);

        write!(expr_file, "{}", self.to_expr(args)).unwrap();

        fs::canonicalize(expr_file.path()).unwrap()
    }
}

pub trait Ask {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized;
}
