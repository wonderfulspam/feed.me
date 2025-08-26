use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use anyhow::{Context, Result};
use clap::Args;

use crate::commands::build::{self, BuildArgs};

#[derive(Args)]
pub struct ServeArgs {
    /// Port to serve on
    #[arg(long, default_value = "8000")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
}

pub fn execute(args: ServeArgs) -> Result<()> {
    // Build the site first
    println!("Building site before serving...");
    let build_args = BuildArgs {
        config_path: args.config_path,
    };
    build::execute(build_args)?;

    // Start the server
    let address = format!("{}:{}", args.host, args.port);
    let listener =
        TcpListener::bind(&address).with_context(|| format!("Failed to bind to {}", address))?;

    println!("🚀 Server running at http://{}/", address);
    println!("Press Ctrl+C to stop");

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let _bytes_read = stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    // Parse the request
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    // Determine file path
    let file_path = if path == "/" {
        "public/index.html".to_string()
    } else if path.ends_with('/') {
        // Directory request, look for index.html
        format!("public{}index.html", path)
    } else {
        // Check if it's a directory without trailing slash
        let dir_path = format!("public{}/index.html", path);
        if Path::new(&dir_path).exists() {
            // Serve the directory's index.html directly
            dir_path
        } else {
            // Direct file request
            format!("public{}", path)
        }
    };

    // Read and serve the file
    if let Ok(contents) = fs::read(&file_path) {
        let content_type = get_content_type(&file_path);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            content_type,
            contents.len()
        );

        stream.write_all(response.as_bytes())?;
        stream.write_all(&contents)?;
    } else {
        // Try 404.html, or send basic 404
        if let Ok(not_found_contents) = fs::read("public/404.html") {
            let response = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                not_found_contents.len()
            );
            stream.write_all(response.as_bytes())?;
            stream.write_all(&not_found_contents)?;
        } else {
            let response = "HTTP/1.1 404 Not Found\r\n\r\nNot Found";
            stream.write_all(response.as_bytes())?;
        }
    }

    Ok(())
}

fn get_content_type(file_path: &str) -> &'static str {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
