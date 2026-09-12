use crate::archive;
use crate::builder;
use crate::config::{ArchiveArgs, Config, UiArgs};
use crate::markdown;
use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

pub const DASHBOARD_HTML: &str = include_str!("../templates/dashboard.html");

pub fn start_server(args: UiArgs) -> Result<()> {
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr)
        .with_context(|| format!("Failed to bind web UI server to {}", addr))?;

    let url = format!("http://{}", addr);
    println!("\n🌐 MDRender Studio UI is running at: {}", url);
    println!("  Active input folder:   {}", args.input.display());
    println!("  Active output folder:  {}", args.output.display());
    println!("  Archive folder:        {}", args.archive_dir.display());
    println!("  Press Ctrl+C to stop.\n");

    if args.open {
        open_browser(&url);
    }

    let shared_args = Arc::new(args);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let args_clone = Arc::clone(&shared_args);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, &args_clone) {
                        // Ignore broken pipe or connection resets from browser closing tabs
                        let _ = e;
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, args: &UiArgs) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let full_path = parts[1];

    // Parse headers
    let mut content_length: usize = 0;
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line)? == 0 {
            break;
        }
        let line = header_line.trim();
        if line.is_empty() {
            break; // Header section ended
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
    }

    // Read body if Content-Length > 0
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body_str = String::from_utf8_lossy(&body);

    let (route, query) = match full_path.split_once('?') {
        Some((r, q)) => (r, q),
        None => (full_path, ""),
    };

    // Route dispatch
    match (method, route) {
        ("GET", "/") | ("GET", "/index.html") => {
            send_response(&mut stream, 200, "text/html; charset=utf-8", DASHBOARD_HTML.as_bytes())?;
        }

        // GET /api/status
        ("GET", "/api/status") => {
            let status_json = get_status_json(&args.input, &args.output, &args.archive_dir)?;
            send_response(&mut stream, 200, "application/json", status_json.as_bytes())?;
        }

        // GET /api/file?path=...
        ("GET", "/api/file") => {
            let rel_path = extract_query_param(query, "path").unwrap_or_default();
            let sanitized = sanitize_rel_path(&rel_path);
            let full_path = args.input.join(sanitized);

            if full_path.exists() && full_path.is_file() {
                let content = fs::read_to_string(&full_path).unwrap_or_default();
                let json = format!(
                    r#"{{"path":"{}","content":"{}"}}"#,
                    json_escape(&rel_path),
                    json_escape(&content)
                );
                send_response(&mut stream, 200, "application/json", json.as_bytes())?;
            } else {
                send_response(&mut stream, 404, "application/json", b"{\"error\":\"File not found\"}")?;
            }
        }

        // POST /api/file
        ("POST", "/api/file") => {
            if let (Some(rel_path), Some(content)) = (
                extract_json_string_field(&body_str, "path"),
                extract_json_string_field(&body_str, "content"),
            ) {
                let sanitized = sanitize_rel_path(&rel_path);
                let target = args.input.join(sanitized);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target, content.as_bytes())?;
                send_response(&mut stream, 200, "application/json", b"{\"success\":true}")?;
            } else {
                send_response(&mut stream, 400, "application/json", b"{\"error\":\"Invalid JSON\"}")?;
            }
        }

        // DELETE /api/file?path=...
        ("DELETE", "/api/file") => {
            let rel_path = extract_query_param(query, "path").unwrap_or_default();
            let sanitized = sanitize_rel_path(&rel_path);
            let full_path = args.input.join(sanitized);

            if full_path.exists() && full_path.is_file() {
                fs::remove_file(full_path)?;
                send_response(&mut stream, 200, "application/json", b"{\"success\":true}")?;
            } else {
                send_response(&mut stream, 404, "application/json", b"{\"error\":\"File not found\"}")?;
            }
        }

        // POST /api/build
        ("POST", "/api/build") => {
            let config = Config {
                ui: false,
                input: args.input.clone(),
                output: args.output.clone(),
                title: args.title.clone(),
                clean: false,
                watch: false,
                init: false,
            };

            match builder::build(&config) {
                Ok(_) => {
                    send_response(&mut stream, 200, "application/json", b"{\"success\":true}")?;
                }
                Err(e) => {
                    let err_json = format!(r#"{{"success":false,"error":"{}"}}"#, json_escape(&e.to_string()));
                    send_response(&mut stream, 500, "application/json", err_json.as_bytes())?;
                }
            }
        }

        // POST /api/archive
        ("POST", "/api/archive") => {
            let name = extract_json_string_field(&body_str, "name").unwrap_or_else(|| "Archive".to_string());
            let next_seq = archive::determine_next_sequence(&args.archive_dir);
            let folder_name = archive::format_archive_folder_name(&name, next_seq);

            let archive_args = ArchiveArgs {
                name,
                archive_dir: args.archive_dir.clone(),
                input: args.input.clone(),
                output: args.output.clone(),
                no_clean: false,
            };

            match archive::run_archive(&archive_args) {
                Ok(_) => {
                    let res_json = format!(r#"{{"success":true,"folder":"{}"}}"#, json_escape(&folder_name));
                    send_response(&mut stream, 200, "application/json", res_json.as_bytes())?;
                }
                Err(e) => {
                    let err_json = format!(r#"{{"success":false,"error":"{}"}}"#, json_escape(&e.to_string()));
                    send_response(&mut stream, 500, "application/json", err_json.as_bytes())?;
                }
            }
        }

        // GET /api/archives
        ("GET", "/api/archives") => {
            let mut list = Vec::new();
            if args.archive_dir.exists() {
                if let Ok(entries) = fs::read_dir(&args.archive_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                list.push(name.to_string());
                            }
                        }
                    }
                }
            }
            list.sort();
            list.reverse(); // newest first

            let items: Vec<String> = list.iter().map(|f| format!(r#"{{"folder":"{}"}}"#, json_escape(f))).collect();
            let json = format!(r#"{{"archives":[{}]}}"#, items.join(","));
            send_response(&mut stream, 200, "application/json", json.as_bytes())?;
        }

        // POST /api/restore
        ("POST", "/api/restore") => {
            if let Some(folder) = extract_json_string_field(&body_str, "folder") {
                let sanitized_folder = sanitize_rel_path(&folder);
                let archived_input = args.archive_dir.join(sanitized_folder).join("input");

                if archived_input.exists() && archived_input.is_dir() {
                    // Empty active input first
                    let _ = archive::clean_directory_contents(&args.input);
                    // Copy archived input to active input
                    archive::copy_dir_all(&archived_input, &args.input)?;
                    send_response(&mut stream, 200, "application/json", b"{\"success\":true}")?;
                } else {
                    send_response(&mut stream, 404, "application/json", b"{\"error\":\"Archive input not found\"}")?;
                }
            } else {
                send_response(&mut stream, 400, "application/json", b"{\"error\":\"Missing folder\"}")?;
            }
        }

        // POST /api/preview
        ("POST", "/api/preview") => {
            let md = extract_json_string_field(&body_str, "markdown").unwrap_or_default();
            let rendered = markdown::render_markdown(&md, "Live Preview");
            let json = format!(r#"{{"html":"{}"}}"#, json_escape(&rendered.html_content));
            send_response(&mut stream, 200, "application/json", json.as_bytes())?;
        }

        // Static file serving: /output/*
        ("GET", p) if p.starts_with("/output/") => {
            let rel = &p["/output/".len()..];
            let sanitized = sanitize_rel_path(rel);
            let file_path = args.output.join(sanitized);

            if file_path.exists() && file_path.is_file() {
                let content = fs::read(&file_path)?;
                let content_type = guess_mime_type(&file_path);
                send_response(&mut stream, 200, content_type, &content)?;
            } else {
                send_response(&mut stream, 404, "text/plain", b"File not found in output")?;
            }
        }

        _ => {
            send_response(&mut stream, 404, "text/plain", b"Not Found")?;
        }
    }

    Ok(())
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) -> Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Status",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status, status_text, content_type, body.len()
    );

    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

fn get_status_json(input_dir: &Path, output_dir: &Path, archive_dir: &Path) -> Result<String> {
    let mut input_files = Vec::new();
    if input_dir.exists() {
        for entry in walkdir::WalkDir::new(input_dir).follow_links(true) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Ok(rel) = entry.path().strip_prefix(input_dir) {
                        let rel_str = rel.to_string_lossy().replace('\\', "/");
                        input_files.push(format!("\"{}\"", json_escape(&rel_str)));
                    }
                }
            }
        }
    }
    input_files.sort();

    let mut archive_count = 0;
    if archive_dir.exists() {
        if let Ok(entries) = fs::read_dir(archive_dir) {
            archive_count = entries.flatten().filter(|e| e.path().is_dir()).count();
        }
    }

    let output_ready = output_dir.join("index.html").exists();

    Ok(format!(
        r#"{{"input_files":[{}],"archive_count":{},"output_ready":{}}}"#,
        input_files.join(","),
        archive_count,
        output_ready
    ))
}

fn guess_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "json" => "application/json",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn sanitize_rel_path(p: &str) -> PathBuf {
    let decoded = urlencoding_decode(p);
    let mut clean = PathBuf::new();
    for comp in Path::new(&decoded).components() {
        match comp {
            std::path::Component::Normal(n) => clean.push(n),
            _ => {}
        }
    }
    clean
}

fn urlencoding_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn extract_query_param(query: &str, key: &str) -> Option<String> {
    for part in query.split('&') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return Some(urlencoding_decode(v));
            }
        }
    }
    None
}

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

pub fn extract_json_string_field(json: &str, field: &str) -> Option<String> {
    let key_pattern = format!("\"{}\"", field);
    let key_idx = json.find(&key_pattern)?;
    let after_key = &json[key_idx + key_pattern.len()..];

    let colon_idx = after_key.find(':')?;
    let after_colon = after_key[colon_idx + 1..].trim_start();

    if !after_colon.starts_with('"') {
        return None;
    }

    let val_content = &after_colon[1..];
    let mut result = String::new();
    let mut chars = val_content.chars();
    let mut escaped = false;

    while let Some(c) = chars.next() {
        if escaped {
            match c {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
                _ => {
                    result.push('\\');
                    result.push(c);
                }
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(result);
        } else {
            result.push(c);
        }
    }

    None
}

fn open_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_escape_and_extract() {
        let raw = "Line 1\nLine 2 with \"quotes\" and \\backslash\\";
        let escaped = json_escape(raw);
        let json = format!(r#"{{"name":"test","content":"{}"}}"#, escaped);

        assert_eq!(extract_json_string_field(&json, "name").as_deref(), Some("test"));
        assert_eq!(extract_json_string_field(&json, "content").as_deref(), Some(raw));
    }

    #[test]
    fn test_sanitize_rel_path() {
        assert_eq!(sanitize_rel_path("guide/intro.md"), PathBuf::from("guide/intro.md"));
        assert_eq!(sanitize_rel_path("../../etc/passwd"), PathBuf::from("etc/passwd"));
        assert_eq!(sanitize_rel_path("/root/file.md"), PathBuf::from("root/file.md"));
    }
}

