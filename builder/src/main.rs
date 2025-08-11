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
extern crate core;

use crate::ConfigError::Inquire;
use builder::{Ask, BuilderConfig};
use clap::Parser;
use inquire::InquireError;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(name = "configurator")]
struct Cli {
    #[arg(long, help = "Uses config file to load/save")]
    config: Option<PathBuf>,

    #[arg(long, help = "Save only saves config")]
    save: bool,
}

#[derive(Error, Debug)]
enum ConfigError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("json parse error")]
    Json(#[from] serde_json::Error),

    #[error("inquire error")]
    Inquire(#[from] InquireError),
}

fn load_config(cli: &Cli) -> Result<BuilderConfig, ConfigError> {
    if let Some(config_path) = &cli.config {
        let data = fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(BuilderConfig::ask()?)
    }
}

fn save_config(config: BuilderConfig, save_path: &PathBuf) -> Result<(), ConfigError> {
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(save_path, json)?;
    Ok(())
}

macro_rules! cancellable {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(Inquire(InquireError::OperationCanceled))
            | Err(Inquire(InquireError::OperationInterrupted)) => return,
            Err(err) => panic!("{err:?}"),
        }
    };
}

fn main() {
    inquire::set_global_render_config(
        RenderConfig::default_colored()
            .with_highlighted_option_prefix(Styled::new(">").with_fg(Color::LightRed))
            .with_answered_prompt_prefix(Styled::new(">").with_fg(Color::DarkRed))
            .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightRed)))
            .with_answer(StyleSheet::empty().with_fg(Color::LightRed))
            .with_help_message(StyleSheet::empty().with_fg(Color::DarkRed))
            .with_selected_checkbox(Styled::new("[x]").with_fg(Color::LightRed))
            .with_prompt_prefix(Styled::new("?").with_fg(Color::LightRed)),
    );

    let cli = Cli::parse();
    let config = cancellable!(load_config(&cli));

    if cli.save {
        let save_path = cli
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from("config.json"));
        cancellable!(save_config(config, &save_path));
        println!("Config saved to {save_path:?}")
    } else {
        config.build()
    }
}
