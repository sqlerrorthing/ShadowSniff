#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::null_mut;
use core::slice;
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::w;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Networking::WinHttp::{WinHttpCloseHandle, WinHttpConnect, WinHttpCrackUrl, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, URL_COMPONENTS, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_FLAG_SECURE};
use utils::WideString;

macro_rules! close {
    ( $( $handle:expr ),* ) => {
        $(
            WinHttpCloseHandle($handle);
        )*
    };
}

pub struct Request {
    method: HttpMethod,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn get(url: String) -> GetBuilder {
        GetBuilder {
            inner: Request {
                method: HttpMethod::GET,
                url,
                headers: BTreeMap::default(),
                body: None
            }
        }
    }

    pub fn post(url: String) -> PostBuilder {
        PostBuilder {
            inner: Request {
                method: HttpMethod::POST,
                url,
                headers: BTreeMap::default(),
                body: None
            }
        }
    }
    
    pub fn send(&self) -> Result<Vec<u8>, u32> {
        unsafe {
            let session = WinHttpOpen(
                w!("PSZAGENTW"),
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                null_mut(),
                null_mut(),
                0
            );
            
            if session.is_null() {
                return Err(GetLastError())
            }
            
            let mut url_comp = URL_COMPONENTS {
                dwStructSize: size_of::<URL_COMPONENTS>() as u32,
                dwSchemeLength: -1i32 as u32,
                dwHostNameLength: -1i32 as u32,
                dwUrlPathLength: -1i32 as u32,
                dwExtraInfoLength: -1i32 as u32,
                ..zeroed()
            };
            
            let url = self.url.to_wide();
            if WinHttpCrackUrl(url.as_ptr(), 0, 0, &mut url_comp) == 0 {
                close!(session);
                return Err(GetLastError());
            }

            let host = slice::from_raw_parts(url_comp.lpszHostName, url_comp.dwHostNameLength as usize);
            let path = slice::from_raw_parts(url_comp.lpszUrlPath, url_comp.dwUrlPathLength as usize);

            let host_str = String::from_utf16_lossy(host);
            let path_str = String::from_utf16_lossy(path);

            let connection = WinHttpConnect(
                session,
                host.as_ptr(),
                url_comp.nPort,
                0,
            );

            if connection.is_null() {
                close!(session);
                return Err(GetLastError());
            }
            
            let method: PCWSTR = self.method.into();

            let request = WinHttpOpenRequest(
                connection,
                method,
                path.as_ptr(),
                null_mut(),
                null_mut(),
                null_mut(),
                WINHTTP_FLAG_SECURE,
            );

            if request.is_null() {
                close!(connection, session);
                return Err(GetLastError());
            }

            let mut body_ptr: *const c_void = null_mut();
            let mut body_len = 0;

            if let Some(ref body) = self.body {
                body_ptr = body.as_ptr() as *const c_void;
                body_len = body.len() as u32;
            }

            let success = WinHttpSendRequest(
                request,
                null_mut(),
                0,
                body_ptr as _,
                body_len,
                body_len,
                0,
            );
            
            if success == 0 {
                close!(request, connection, session);
                return Err(GetLastError());
            }
            
            if WinHttpReceiveResponse(request, null_mut()) == 0 {
                close!(request, connection, session);
                return Err(GetLastError());
            }

            let mut result: Vec<u8> = Vec::new();
            
            loop {
                let mut bytes_available: u32 = 0;
                if WinHttpQueryDataAvailable(request, &mut bytes_available) == 0 {
                    break;
                }
                if bytes_available == 0 {
                    break;
                }

                let mut buffer = vec![0u8; bytes_available as usize];
                let mut bytes_read = 0;

                if WinHttpReadData(
                    request,
                    buffer.as_mut_ptr() as _,
                    bytes_available,
                    &mut bytes_read,
                ) == 0
                {
                    break;
                }

                buffer.truncate(bytes_read as usize);
                result.extend_from_slice(&buffer);
            }

            close!(request, connection, session);
            
            Ok(result)
        }
    }
}

pub trait RequestBuilder {
    fn header<S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: AsRef<str>;

    fn build(self) -> Request;
}

pub trait BodyRequestBuilder: RequestBuilder {
    fn body<B>(&mut self, body: B) -> &mut Self
    where
        B: Into<Vec<u8>>;
}

#[derive(Copy, Clone)]
pub enum HttpMethod {
    GET,
    POST
}

impl From<HttpMethod> for PCWSTR {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::GET => w!("GET\0"),
            HttpMethod::POST => w!("POST\0")
        }
    }
}

impl RequestBuilder for Request {
    fn header<S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: AsRef<str>
    {
        self.headers.insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    fn build(self) -> Request {
        self
    }
}

pub struct GetBuilder {
    inner: Request
}

impl RequestBuilder for GetBuilder {
    fn header<S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: AsRef<str>
    {
        self.inner.header(key, value);
        self
    }

    fn build(self) -> Request {
        self.inner
    }
}

pub struct PostBuilder {
    inner: Request
}

impl RequestBuilder for PostBuilder {
    fn header<S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: AsRef<str>
    {
        self.inner.header(key, value);
        self
    }

    fn build(self) -> Request {
        self.inner
    }
}

impl BodyRequestBuilder for PostBuilder {
    fn body<B>(&mut self, body: B) -> &mut Self
    where
        B: Into<Vec<u8>>
    {
        self.inner.body = Some(body.into());
        self
    }
}