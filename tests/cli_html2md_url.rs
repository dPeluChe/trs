//! `trs html2md <url>` against a local server, so the fetch path runs in CI
//! with no network.

use assert_cmd::Command;
use std::io::{Read, Write};
use std::net::TcpListener;

const PAGE: &str = "<html><body><h1>Moby-Dick</h1><p>Herman Melville</p></body></html>";

fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/page", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{PAGE}",
                PAGE.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    url
}

fn html2md(args: &[&str]) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .env("NO_PROXY", "127.0.0.1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_fetched_page_becomes_markdown() {
    let url = serve();
    let text = html2md(&["html2md", &url]);
    assert!(
        text.contains("Moby-Dick") && text.contains("Herman Melville"),
        "{text}"
    );

    let text = html2md(&["html2md", &url, "--metadata"]);
    assert!(
        text.contains("source") && text.contains("127.0.0.1"),
        "{text}"
    );

    let text = html2md(&["--json", "html2md", &url]);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(json["markdown"].as_str().unwrap().contains("Moby-Dick"));
}
