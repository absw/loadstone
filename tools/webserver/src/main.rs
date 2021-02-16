use std::{
    path::PathBuf,
    sync::Mutex,
};
use futures::{SinkExt, StreamExt};
use warp::{
    Filter,
    http::StatusCode,
    reply::Response,
    ws::{WebSocket, Message}
};
use serial::SystemPort;

#[macro_use]
extern crate lazy_static;

fn try_parse_metrics(string: &str) -> Option<String> {
    // TODO: Clean up this function.
    const REGEX_SOURCE : &str =
        r#"\[Boot Metrics\][\r\n]+\* (.*)[\r\n]+\* Boot process took (.*) milliseconds\."#;
    let regex = regex::Regex::new(REGEX_SOURCE).unwrap();
    let captures = regex.captures(string)?;
    let path = captures.get(1)?.as_str().trim();
    let time = captures.get(2)?.as_str();

    Some(format!(
        r#"{{ "error": "none", "path": "{}", "time": "{}ms" }}"#,
        path, time,
    ))
}

fn respond_to_api_request(file_name: String) -> Response {
    match file_name.as_str() {
        "server-version" => {
            Response::new(std::env!("CARGO_PKG_VERSION").into())
        },
        "metrics" => {
            let mut device = match setup_device() {
                None => return Response::new(r#"{ "error": "device", "path": "", "time": "" }"#.into()),
                Some(d) => d,
            };

            let mut try_read = || {
                write(&mut device, b"metrics\n").ok()?;
                read(&mut device).ok()
            };

            let raw = match try_read() {
                None => return Response::new(r#"{ "error": "io", "path": "", "time": "" }"#.into()),
                Some(r) => r,
            };

            let message = String::from_utf8_lossy(&raw);

            match try_parse_metrics(&message) {
                None => Response::new(r#"{ "error": "metrics", "path": "", "time": "" }"#.into()),
                Some(m) => Response::new(m.into()),
            }
        },
        _ => {
            let mut response = Response::new("404 Not found".into());
            let status = response.status_mut();
            *status = StatusCode::NOT_FOUND;
            response
        },
    }
}

fn write(serial: &mut SystemPort, buffer: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut remaining = buffer.len();
    while remaining > 0 {
        let to_write = &buffer[(buffer.len() - remaining)..];
        match serial.write(to_write) {
            Ok(n) => {
                remaining -= n
            },
            Err(e) if (e.kind() == std::io::ErrorKind::Interrupted) => {},
            Err(e) => {
                return Err(e);
            },
        }
    }

    Ok(())
}

fn read(serial: &mut SystemPort) -> std::io::Result<Vec<u8>> {
    use std::io::Read;

    let mut buffer = Vec::<u8>::new();

    loop {
        let mut byte = [0u8];
        match serial.read(&mut byte) {
            Ok(0) => {
                return Ok(buffer);
            },
            Ok(_) => {
                if byte[0] == b'\n' {
                    buffer.push(b'\r');
                }
                buffer.push(byte[0]);
            },
            Err(e) if (e.kind() == std::io::ErrorKind::Interrupted) => {},
            Err(e) if (e.kind() == std::io::ErrorKind::TimedOut) => {
                return Ok(buffer);
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}

lazy_static! {
    static ref SERIAL : Mutex<SystemPort> = Mutex::new(setup_device().unwrap());
}

async fn handle_websocket(socket: WebSocket) {
    // TODO: Tell client that the websocket is about to close before closing.
    // TODO: Do something better than calling `unwrap`.

    let (mut sender, mut reciever) = socket.split();

    while let Some(request) = reciever.next().await {
        let message = match request {
            Ok(message) => message,
            Err(_) => { Message::close() },
        };

        if message.is_close() {
            sender.close().await.unwrap();
            return;
        }

        let buffer = {
            let serial : &mut SystemPort = &mut SERIAL.lock().unwrap();
            write(serial, message.as_bytes()).unwrap();
            write(serial, &[b'\n']).unwrap();
            read(serial).unwrap()
        };

        let response = Message::text(String::from_utf8_lossy(&buffer));

        sender.send(response).await.unwrap();
    }
}

pub fn setup_device() -> Option<SystemPort> {
    use serial::*;

    let mut device = SystemPort::open(&PathBuf::from("/dev/ttyUSB0")).ok()?;
    device.reconfigure(&|s| {
        s.set_baud_rate(Baud115200)?;
        s.set_char_size(Bits8);
        s.set_parity(ParityNone);
        s.set_stop_bits(Stop1);
        s.set_flow_control(FlowNone);
        Ok(())
    }).ok()?;
    Some(device)
}

#[tokio::main]
async fn main() {
    let html_directory : PathBuf = PathBuf::from("public_html/");

    let get_request = warp::get();

    let index = get_request
        .and(warp::path::end())
        .and(warp::fs::file(html_directory.join("index.html")));

    let files = get_request
        .and(warp::fs::dir(html_directory));

    let api_request = get_request
        .and(warp::path!("api" / String))
        .map(respond_to_api_request);

    let serial_websocket = warp::ws()
        .and(warp::path!("serial"))
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_websocket));

    let not_found = get_request
        .map(|| {
            let mut response = Response::new("404 Not found".into());
            let status = response.status_mut();
            *status = StatusCode::NOT_FOUND;
            response
        });

    let routes = index
        .or(api_request)
        .or(files)
        .or(serial_websocket)
        .or(not_found);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8000))
        .await;
}
