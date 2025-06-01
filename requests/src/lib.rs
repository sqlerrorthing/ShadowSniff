#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::{null, null_mut};
use core::slice;
use json::{parse, ParseError, Value};
use utils::WideString;
use windows_sys::core::PCWSTR;
use windows_sys::w;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER};
use windows_sys::Win32::Networking::WinHttp::{WinHttpAddRequestHeaders, WinHttpCloseHandle, WinHttpConnect, WinHttpCrackUrl, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable, WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, URL_COMPONENTS, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_ADDREQ_FLAG_ADD, WINHTTP_FLAG_SECURE, WINHTTP_INTERNET_SCHEME_HTTPS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_RAW_HEADERS_CRLF, WINHTTP_QUERY_STATUS_CODE};

macro_rules! close {
    ( $( $handle:expr ),* ) => {
        $(
            WinHttpCloseHandle($handle);
        )*
    };
}

pub type ResponseBody = Vec<u8>;

pub trait ResponseBodyExt {
    fn as_json(&self) -> Result<Value, ParseError>;
}

impl ResponseBodyExt for ResponseBody {
    fn as_json(&self) -> Result<Value, ParseError> {
        parse(self)
    }
}

pub struct Request {
    method: HttpMethod,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<Vec<u8>>,
}

pub struct Response {
    status_code: u16,
    headers: BTreeMap<String, String>,
    body: ResponseBody,
}

impl Response {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &ResponseBody {
        &self.body
    }
}

impl Request {
    pub fn get<S>(url: S) -> GetBuilder
    where
        S: Into<String>
    {
        GetBuilder {
            inner: Request {
                method: HttpMethod::GET,
                url: url.into(),
                headers: BTreeMap::default(),
                body: None
            }
        }
    }

    pub fn post<S>(url: S) -> PostBuilder
    where
        S: Into<String>
    {
        PostBuilder {
            inner: Request {
                method: HttpMethod::POST,
                url: url.into(),
                headers: BTreeMap::default(),
                body: None
            }
        }
    }

    pub fn send(&self) -> Result<Response, u32> {
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

            let mut host = slice::from_raw_parts(url_comp.lpszHostName, url_comp.dwHostNameLength as usize).to_vec();
            host.push(0);

            let mut path = slice::from_raw_parts(url_comp.lpszUrlPath, url_comp.dwUrlPathLength as usize).to_vec();
            path.push(0);

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
                if url_comp.nScheme == WINHTTP_INTERNET_SCHEME_HTTPS { WINHTTP_FLAG_SECURE } else { 0 },
            );

            if request.is_null() {
                close!(connection, session);
                return Err(GetLastError());
            }

            for (key, value) in &self.headers {
                let header = alloc::format!("{}: {}\0", key, value);
                let header_wide: Vec<u16> = header.encode_utf16().collect();
                if WinHttpAddRequestHeaders(request, header_wide.as_ptr(), header.len() as u32, WINHTTP_ADDREQ_FLAG_ADD) == 0 {
                    close!(request, connection, session);
                    return Err(GetLastError());
                }
            }

            let (body_ptr, body_len) = match &self.body {
                Some(b) => (b.as_ptr(), b.len() as u32),
                None => (null(), 0),
            };

            if WinHttpSendRequest(request, null(), 0, body_ptr as _, body_len, body_len, 0) == 0 {
                close!(request, connection, session);
                return Err(GetLastError());
            }

            if WinHttpReceiveResponse(request, null_mut()) == 0 {
                close!(request, connection, session);
                return Err(GetLastError());
            }

            let mut status_code: u32 = 0;
            let mut size = size_of::<u32>() as u32;
            if WinHttpQueryHeaders(
                request,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                null(),
                &mut status_code as *mut _ as *mut _,
                &mut size,
                null_mut(),
            ) == 0 {
                close!(request, connection, session);
                return Err(GetLastError());
            }

            let mut headers = BTreeMap::new();
            let mut buffer_len: u32 = 0;
            let result = WinHttpQueryHeaders(
                request,
                WINHTTP_QUERY_RAW_HEADERS_CRLF,
                null(),
                null_mut(),
                &mut buffer_len,
                null_mut(),
            );

            if result == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                let mut buffer: Vec<u16> = vec![0; buffer_len as usize / 2];
                if WinHttpQueryHeaders(
                    request,
                    WINHTTP_QUERY_RAW_HEADERS_CRLF,
                    null(),
                    buffer.as_mut_ptr() as *mut _,
                    &mut buffer_len,
                    null_mut(),
                ) != 0
                {
                    let headers_str = String::from_utf16_lossy(&buffer[..(buffer_len as usize / 2)]);
                    for line in headers_str.lines().skip(1) {
                        if let Some(colon_pos) = line.find(':') {
                            let key = line[..colon_pos].trim().to_string();
                            let value = line[colon_pos + 1..].trim().to_string();
                            headers.insert(key, value);
                        }
                    }
                }
            }

            let mut body = Vec::new();
            loop {
                let mut bytes_available: u32 = 0;
                if WinHttpQueryDataAvailable(request, &mut bytes_available) == 0 || bytes_available == 0 {
                    break;
                }

                let mut buffer = vec![0u8; bytes_available as usize];
                let mut bytes_read = 0;
                if WinHttpReadData(request, buffer.as_mut_ptr() as _, bytes_available, &mut bytes_read) == 0 || bytes_read == 0 {
                    break;
                }

                buffer.truncate(bytes_read as usize);
                body.extend_from_slice(&buffer);
            }

            close!(request, connection, session);

            Ok(Response {
                status_code: status_code as u16,
                headers,
                body,
            })
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