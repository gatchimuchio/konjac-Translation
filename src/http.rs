use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use konjac_translation::{MAX_SOURCE_BYTES, TranslationSubmission, execute, mode_capabilities};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = MAX_SOURCE_BYTES + 8 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub fn serve(bind: SocketAddr) -> io::Result<()> {
    if !bind.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Konjac local UI may bind only to a loopback address",
        ));
    }
    let listener = TcpListener::bind(bind)?;
    let address = listener.local_addr()?;
    eprintln!("Konjac local UI: http://{address}/");
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream, address) {
                    eprintln!("Konjac local request ended: {error}");
                }
            }
            Err(error) => eprintln!("Konjac local listener error: {error}"),
        }
    }
    Ok(())
}

struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpError(u16, &'static str);

fn handle_connection(mut stream: TcpStream, address: SocketAddr) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.set_write_timeout(Some(READ_TIMEOUT))?;
    let request = match read_request(&mut BufReader::new(&mut stream)) {
        Ok(request) => request,
        Err(error) => {
            return respond(
                &mut stream,
                error.0,
                error.1,
                "text/plain; charset=utf-8",
                b"\n",
            );
        }
    };
    if !valid_host(request.headers.get("host"), address) {
        return respond(
            &mut stream,
            400,
            "Bad Request",
            "text/plain; charset=utf-8",
            b"\n",
        );
    }
    if !valid_origin(
        request.headers.get("origin"),
        request.headers.get("host"),
        address,
    ) {
        return respond(
            &mut stream,
            403,
            "Forbidden",
            "text/plain; charset=utf-8",
            b"\n",
        );
    }
    route(&mut stream, request)
}

fn read_request(reader: &mut impl BufRead) -> Result<Request, HttpError> {
    let mut raw_header = Vec::new();
    loop {
        let room = MAX_HEADER_BYTES.saturating_sub(raw_header.len());
        if room == 0 {
            return Err(HttpError(431, "Request Header Fields Too Large"));
        }
        let mut line = Vec::new();
        reader
            .take(room as u64)
            .read_until(b'\n', &mut line)
            .map_err(|_| HttpError(400, "Bad Request"))?;
        if line.is_empty() || !line.ends_with(b"\r\n") {
            return Err(HttpError(400, "Bad Request"));
        }
        raw_header.extend_from_slice(&line);
        if line == b"\r\n" {
            break;
        }
    }
    let header = std::str::from_utf8(&raw_header).map_err(|_| HttpError(400, "Bad Request"))?;
    let mut lines = header.split("\r\n");
    let request_line = lines.next().ok_or(HttpError(400, "Bad Request"))?;
    let mut request_parts = request_line.split(' ');
    let method = request_parts.next().unwrap_or_default();
    let target = request_parts.next().unwrap_or_default();
    let version = request_parts.next().unwrap_or_default();
    if request_parts.next().is_some()
        || method.is_empty()
        || !method.bytes().all(|byte| byte.is_ascii_uppercase())
        || target.is_empty()
        || !target.starts_with('/')
        || target.bytes().any(|byte| byte <= 32 || byte == 127)
        || version != "HTTP/1.1"
    {
        return Err(HttpError(400, "Bad Request"));
    }
    let mut headers = BTreeMap::new();
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or(HttpError(400, "Bad Request"))?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || value
                .bytes()
                .any(|byte| (byte < 32 && byte != b'\t') || byte == 127)
        {
            return Err(HttpError(400, "Bad Request"));
        }
        if headers
            .insert(name.to_ascii_lowercase(), value.trim().to_owned())
            .is_some()
        {
            return Err(HttpError(400, "Bad Request"));
        }
    }
    if headers.contains_key("transfer-encoding") || headers.contains_key("expect") {
        return Err(HttpError(400, "Bad Request"));
    }
    if !headers.contains_key("host") {
        return Err(HttpError(400, "Bad Request"));
    }
    let content_length = match headers.get("content-length") {
        Some(value) if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
            value
                .parse::<usize>()
                .map_err(|_| HttpError(413, "Content Too Large"))?
        }
        Some(_) => return Err(HttpError(400, "Bad Request")),
        None => 0,
    };
    if content_length > MAX_BODY_BYTES {
        return Err(HttpError(413, "Content Too Large"));
    }
    if method == "POST" && !headers.contains_key("content-length") {
        return Err(HttpError(411, "Length Required"));
    }
    let mut body = vec![0; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|_| HttpError(400, "Bad Request"))?;
    Ok(Request {
        method: method.to_owned(),
        target: target.to_owned(),
        headers,
        body,
    })
}

fn valid_host(host: Option<&String>, address: SocketAddr) -> bool {
    let Some(host) = host else { return false };
    match address.ip() {
        IpAddr::V4(ip) => {
            host.as_str() == format!("{ip}:{}", address.port())
                || (ip.is_loopback() && host.as_str() == format!("localhost:{}", address.port()))
        }
        IpAddr::V6(ip) => {
            host.as_str() == format!("[{ip}]:{}", address.port())
                || (ip.is_loopback() && host.as_str() == format!("localhost:{}", address.port()))
        }
    }
}

fn valid_origin(origin: Option<&String>, host: Option<&String>, address: SocketAddr) -> bool {
    let Some(origin) = origin else { return true };
    let Some(host) = host else { return false };
    origin.as_str() == format!("http://{host}")
        || (address.ip().is_loopback()
            && origin.as_str() == format!("http://localhost:{}", address.port()))
}

fn route(stream: &mut TcpStream, request: Request) -> io::Result<()> {
    match (request.method.as_str(), request.target.as_str()) {
        ("GET", "/") => respond(
            stream,
            200,
            "OK",
            "text/html; charset=utf-8",
            include_bytes!("../web/index.html"),
        ),
        ("GET", "/assets/app.css") => respond(
            stream,
            200,
            "OK",
            "text/css; charset=utf-8",
            include_bytes!("../web/app.css"),
        ),
        ("GET", "/assets/app.js") => respond(
            stream,
            200,
            "OK",
            "text/javascript; charset=utf-8",
            include_bytes!("../web/app.js"),
        ),
        ("GET", "/api/v1/capabilities") => json_response(stream, 200, &mode_capabilities()),
        ("POST", "/api/v1/translate") => {
            if request.headers.get("content-type").map(String::as_str) != Some("application/json") {
                return respond(stream, 415, "Unsupported Media Type", "application/json; charset=utf-8", br#"{"error":{"code":"UNSUPPORTED_MEDIA_TYPE","message":"Content-Type must be application/json"}}"#);
            }
            let submission: TranslationSubmission = match serde_json::from_slice(&request.body) {
                Ok(value) => value,
                Err(_) => return respond(stream, 400, "Bad Request", "application/json; charset=utf-8", br#"{"error":{"code":"INVALID_JSON","message":"Request body must match the documented JSON contract"}}"#),
            };
            match execute(submission) {
                Ok(response) => json_response(stream, 200, &response),
                Err(error) => {
                    let body =
                        serde_json::json!({"error":{"code":error.code,"message":error.message}});
                    json_response(stream, 422, &body)
                }
            }
        }
        ("GET", _) => respond(
            stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"Not found\n",
        ),
        ("POST", _) => respond(
            stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"Not found\n",
        ),
        _ => respond(
            stream,
            405,
            "Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Method not allowed\n",
        ),
    }
}

fn json_response<T: serde::Serialize>(
    stream: &mut TcpStream,
    status: u16,
    value: &T,
) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    let reason = if status == 200 {
        "OK"
    } else if status == 422 {
        "Unprocessable Content"
    } else {
        "Internal Server Error"
    };
    respond(
        stream,
        status,
        reason,
        "application/json; charset=utf-8",
        &body,
    )
}

fn respond(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    let security = concat!(
        "Cache-Control: no-store\r\n",
        "X-Content-Type-Options: nosniff\r\n",
        "Referrer-Policy: no-referrer\r\n",
        "X-Frame-Options: DENY\r\n",
        "Permissions-Policy: camera=(), microphone=(), geolocation=()\r\n",
        "Content-Security-Policy: default-src 'none'; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'; object-src 'none'\r\n",
    );
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n{security}\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn parser_rejects_chunked_and_duplicate_headers() {
        let chunked = b"POST /api/v1/translate HTTP/1.1\r\nHost: 127.0.0.1:8787\r\nTransfer-Encoding: chunked\r\n\r\n";
        assert!(read_request(&mut &chunked[..]).is_err());
        let duplicate = b"GET / HTTP/1.1\r\nHost: 127.0.0.1:8787\r\nHost: localhost:8787\r\n\r\n";
        assert!(read_request(&mut &duplicate[..]).is_err());
    }

    #[test]
    fn rejects_cross_origin_and_non_loopback_hosts() {
        let address: SocketAddr = "127.0.0.1:8787".parse().unwrap();
        assert!(!valid_origin(
            Some(&"http://evil.example".into()),
            Some(&"127.0.0.1:8787".into()),
            address
        ));
        assert!(valid_host(Some(&"localhost:8787".into()), address));
        assert!(!valid_host(Some(&"127.0.0.1:9999".into()), address));
    }

    #[test]
    fn normal_http_entry_returns_a_blocker_without_echoing_source_text() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, address).unwrap();
        });
        let source = "PRIVATE-SOURCE-DO-NOT-ECHO";
        let body = serde_json::json!({
            "contract_version": "konjac.translation.v1",
            "session_id": "session-http-1",
            "input_version": 1,
            "source_kind": "text",
            "source_text": source,
            "source_language_hint": "en",
            "target_languages": ["ja", "fr"]
        })
        .to_string();
        let mut client = TcpStream::connect(address).unwrap();
        write!(
            client,
            "POST /api/v1/translate HTTP/1.1\r\nHost: {}\r\nOrigin: http://{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            address,
            address,
            body.len(),
            body
        )
        .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response.contains("BLOCKED_ENGINE_CAPABILITY"));
        assert!(response.contains("\"variants\":[]"));
        assert!(response.contains("\"target_language\":\"ja\""));
        assert!(response.contains("\"target_language\":\"fr\""));
        assert!(!response.contains(source));
    }
}
