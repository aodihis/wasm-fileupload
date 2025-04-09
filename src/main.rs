use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use tiny_http::{Server, Method, Response, Header};
use chrono::Local;

fn main() {
    let server = Server::http("0.0.0.0:7000").unwrap();
    println!("Server is running on http://localhost:7000");

    for mut request in server.incoming_requests() {
        let method = request.method();
        let url = request.url().to_string();

        if method == &Method::Options && url == "/upload" {
            let mut response = Response::empty(200);
            for header in cors_headers_for_options() {
                response.add_header(header);
            }
            let _ = request.respond(response);
            continue;
        }

        if method == &Method::Get && url == "/" {
            let path = "index.html";
            match fs::read_to_string(path) {
                Ok(html) => {
                    let response = Response::from_string(html)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let response = Response::from_string("404 Not Found")
                        .with_status_code(404);
                    let _ = request.respond(response);
                }
            }
            continue
        }

        let allowed_folders = ["assets/", "pkg/", "static/"];
        let path = &url[1..]; // trim leading slash
        if allowed_folders.iter().any(|prefix| path.starts_with(prefix)) {
            let file_path = Path::new(path);
            if let Ok(mut file) = File::open(file_path) {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();

                let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    Some("wasm") => "application/wasm",
                    Some("html") => "text/html",
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    _ => "application/octet-stream",
                };

                let response = Response::from_data(contents)
                    .with_header(Header::from_bytes("Content-Type", content_type).unwrap());
                let _ = request.respond(response);
            } else {
                let _ = request.respond(Response::from_string("404").with_status_code(404));
            }
            continue;
        }
        if  method == &Method::Post && url == "/upload" {

            // Read the body
            let mut body = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body) {
                let response = Response::from_string(format!("Failed to read body: {}", e)).with_header(cors_header())
                    .with_status_code(500);
                let _ = request.respond(response);
                continue;
            }

            let now = Local::now().format("%Y%m%d%H%M%S").to_string();

            let ori_filename = body
                .lines()
                .find(|line| line.contains("Content-Disposition") && line.contains("filename="))
                .and_then(|line| {
                    line.split(';')
                        .find_map(|part| {
                            let part = part.trim();
                            if part.starts_with("filename=") {
                                Some(part.trim_start_matches("filename=").trim_matches('"').to_string())
                            } else {
                                None
                            }
                        })
                })
                .unwrap_or_else(|| "upload_default.bin".to_string());

            let filename = format!("{}-{}", now, ori_filename);

            let folder = "uploads";
            fs::create_dir_all(folder).unwrap();

            let filepath = format!("{}/{}", folder, filename);
            let file = File::create(&filepath).expect("Failed to create file");
            let mut f = BufWriter::new(file);
            f.write_all(&body.as_bytes()).unwrap();


            let response = Response::from_string("JSON received and saved").with_header(cors_header());
            let _ = request.respond(response);
        } else {
            // Method not allowed or wrong path
            let response = Response::from_string("Not Found")
                .with_status_code(404);
            let _ = request.respond(response);
        }
    }
}

fn cors_header() -> Header {
    Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap()
}

fn cors_headers_for_options() -> Vec<Header> {
    vec![
        Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
        Header::from_bytes("Access-Control-Allow-Methods", "POST, OPTIONS").unwrap(),
        Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap(),
    ]
}
