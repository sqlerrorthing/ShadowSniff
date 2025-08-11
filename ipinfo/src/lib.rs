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

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use derive_new::new;
use indoc::writedoc;
use json::Value;
use lazy_static::lazy_static;
use obfstr::obfstr as s;
use requests::{Request, RequestBuilder, ResponseBodyExt};
use spin::Mutex;
use utils::internal_code_to_flag;

lazy_static! {
    static ref GLOBAL_IP_INFO: Mutex<Option<IpInfo>> = Mutex::new(None);
}

#[allow(clippy::too_many_arguments)]
#[derive(new, Clone)]
pub struct IpInfo {
    #[new(into)]
    pub ip: Arc<str>,
    #[new(into)]
    pub city: Arc<str>,
    #[new(into)]
    pub region: Arc<str>,
    #[new(into)]
    pub country: Arc<str>,
    #[new(into)]
    pub loc: Arc<str>,
    #[new(into)]
    pub org: Arc<str>,
    #[new(into)]
    pub timezone: Arc<str>,
}

impl Display for IpInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writedoc!(
            f,
            "
            IP: {}
            \tCity:\t({}) {}
            \tRegion:\t{}",
            self.ip,
            internal_code_to_flag(&self.country)
                .map(Arc::from)
                .unwrap_or(self.country.clone()),
            self.city,
            self.region,
        )
    }
}

#[allow(static_mut_refs)]
pub fn get_ip_info() -> Option<IpInfo> {
    GLOBAL_IP_INFO.lock().clone()
}

pub fn unwrapped_ip_info() -> IpInfo {
    get_ip_info().unwrap()
}

fn init_ip_info(api: &str, mapper: impl FnOnce(Value) -> Option<IpInfo>) -> Option<bool> {
    if get_ip_info().is_some() {
        return None;
    }

    let result = Request::get(api)
        .header("Accept", "application/json")
        .build()
        .send();

    let Some(info) = result.ok()
        .and_then(|response| response.body().as_json().ok())
        .and_then(mapper)
    else {
        return Some(false);
    };

    GLOBAL_IP_INFO.lock().replace(info);

    Some(true)
}

#[allow(static_mut_refs)]
fn init_ip_info_io() -> Option<bool> {
    init_ip_info(s!("https://ipinfo.io/json"), |value| {
        Some(IpInfo::new(
            value.get(s!("ip"))?.as_string()?,
            value.get(s!("city"))?.as_string()?,
            value.get(s!("region"))?.as_string()?,
            value.get(s!("country"))?.as_string()?,
            value.get(s!("loc"))?.as_string()?,
            value.get(s!("org"))?.as_string()?,
            value.get(s!("timezone"))?.as_string()?,
        ))
    })
}

fn init_ip_ip_wtf() -> Option<bool> {
    init_ip_info(s!("https://ip.wtf"), |value| {
        let location = value.get(s!("location"))?;
        let r#as = value.get(s!("as"))?;

        let loc = format!(
            "{},{}",
            location.get(s!("latitude"))?.as_number()?,
            location.get(s!("longitude"))?.as_number()?,
        );

        let org = format!(
            "AS{} {}",
            r#as.get("number")?.as_number()?,
            r#as.get("name")?.as_string()?,
        );

        Some(IpInfo::new(
            value.get(s!("ip"))?.as_string()?,
            location.get(s!("city"))?.as_string()?,
            location.get(s!("region_name"))?.as_string()?,
            location.get(s!("country"))?.as_string()?,
            loc,
            org,
            location.get(s!("timezone"))?.get(s!("name"))?.as_string()?,
        ))
    })
}

pub fn init() -> Option<()> {
    if init_ip_ip_wtf() != Some(true)
        && init_ip_info_io() != Some(true)
    {
        return None;
    }

    Some(())
}