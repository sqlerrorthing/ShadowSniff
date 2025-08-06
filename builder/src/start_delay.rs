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
use crate::{Ask, ToExpr};
use inquire::{CustomType, InquireError};
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::ops::RangeInclusive;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone)]
pub enum StartDelay {
    Fixed(u32),
    Random(RangeInclusive<u32>),
    None,
}

impl Display for StartDelay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartDelay::Fixed(fixed) => write!(f, "{fixed}ms"),
            StartDelay::Random(range) => {
                let (start, end) = (range.start(), range.end());
                write!(f, "{start}..{end}ms")
            }
            StartDelay::None => write!(f, "As fast as possible"),
        }
    }
}

impl ToExpr for StartDelay {
    fn to_expr(&self, _args: ()) -> TokenStream {
        match self {
            StartDelay::Fixed(value) => quote! {
                unsafe {
                    windows_sys::Win32::System::Threading::Sleep(#value);
                }
            },
            StartDelay::Random(value) => {
                let (start, end) = (value.start(), value.end());

                quote! {
                    unsafe {
                        let range = #end - #start + 1;

                        let mut rng: rand_chacha::ChaCha20Rng = utils::random::ChaCha20RngExt::from_nano_time();
                        let rand_u32 = rng.next_u32();
                        let scaled = rand_u32 % range;
                        windows_sys::Win32::System::Threading::Sleep(#start + scaled);
                    }
                }
            }
            StartDelay::None => quote! {{}},
        }
    }
}

#[derive(Error, Debug)]
pub enum StartDelayParseError {
    #[error("Not a valid number")]
    NaN(#[from] ParseIntError),

    #[error("Start value {start} is greater than end value {end}")]
    StartGreaterThanEnd { start: u32, end: u32 },
}

impl FromStr for StartDelay {
    type Err = StartDelayParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::None);
        }

        if let Some(pos) = s.find("..") {
            let start_str = &s[..pos];
            let end_str = &s[pos + 2..];

            let start = start_str.parse::<u32>()?;
            let end = end_str.parse::<u32>()?;

            if start > end {
                return Err(StartDelayParseError::StartGreaterThanEnd { start, end });
            }

            return if start == end {
                Ok(Self::Fixed(start))
            } else {
                Ok(Self::Random(start..=end))
            };
        }

        let fixed = s.parse::<u32>()?;
        Ok(Self::Fixed(fixed))
    }
}

impl Ask for StartDelay {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized,
    {
        let ans = CustomType::<Self>::new("What is the start delay?")
            .with_help_message("Leave empty for no delay. Enter a number (ms) for a fixed delay, or a range like '100..200' (ms, inclusive) for a random delay within that range.")
            .prompt_skippable()?;

        match ans {
            Some(ans) => Ok(ans),
            None => Ok(Self::None),
        }
    }
}
