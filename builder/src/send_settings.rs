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
use std::fmt::{Display, Formatter};
use derive_new::new;
use inquire::{Confirm, InquireError, Select};
use proc_macro2::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use crate::{Ask, ToExpr};
use crate::sender_service::SenderService;

pub type Uploader = (UploaderService, UploaderUsecase);

#[derive(Display, EnumIter, Clone)]
pub enum UploaderService {
    Gofile,
    TmpFiles,
    Catbox
}

impl Ask for UploaderService {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new("Which external storage provider would you like to use?", UploaderService::iter().collect())
            .prompt()
    }
}

impl ToExpr<(TokenStream,)> for UploaderService {
    fn to_expr(&self, args: (TokenStream,)) -> TokenStream {
        let (base,) = args;

        match self {
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
}

#[derive(EnumIter, Clone)]
pub enum UploaderUsecase {
    Always,
    WhenLogExceedsLimit
}

impl Display for UploaderUsecase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            UploaderUsecase::Always => write!(f, "Always"),
            UploaderUsecase::WhenLogExceedsLimit => write!(f, "When log exceeds service filesize limit"),
        }
    }
}

impl Ask for UploaderUsecase {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new("Under what condition should the log be uploaded to external storage?", UploaderUsecase::iter().collect())
            .prompt()
    }
}

#[derive(new)]
pub struct SendSettings {
    pub(crate) service: SenderService,
    pub(crate) uploader: Option<Uploader>,
}

impl Ask for Option<Uploader> {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let r#use = Confirm::new("Do you want to use external storage for the log file?")
            .with_default(true)
            .with_help_message("This allows you to send very large logs if the log exceeds the service's filesize limit.")
            .prompt()?;

        if !r#use {
            return Ok(None);
        }

        let service = UploaderService::ask()?;
        let usecase = UploaderUsecase::ask()?;
        Ok(Some((service, usecase)))
    }
}

impl Ask for SendSettings {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let service = SenderService::ask()?;
        println!();

        let uploader = Option::<Uploader>::ask()?;

        Ok(Self::new(service, uploader))
    }
}