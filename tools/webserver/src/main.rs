use std::path::PathBuf;
use serial::SystemPort;
use warp::{
    Filter,
    http::StatusCode,
    reply::Response,
};

enum MetricsError {
    BadPath,
    BadDevice,
    WriteError,
    ReadError,
    BadMetrics
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MetricsError::*;
        match self {
            BadPath => write!(f, "internal"),
            BadDevice => write!(f, "device"),
            WriteError => write!(f, "io"),
            ReadError => write!(f, "io"),
            BadMetrics => write!(f, "metrics"),
        }
    }
}

fn get_device_path() -> Option<String> {
    // The device path should be the first argument passed to the server.
    std::env::args().nth(1)
}

fn try_parse_metrics(string: &str) -> Option<(String, String)> {
    const REGEX_SOURCE : &str =
        r#"\[Boot Metrics\][\r\n]+\* (.*)[\r\n]+\* Boot process took (.*) milliseconds\."#;
    let regex = regex::Regex::new(REGEX_SOURCE).unwrap();
    let captures = regex.captures(string)?;
    let path = captures.get(1)?.as_str().trim();
    let time = captures.get(2)?.as_str();

    Some((path.into(), time.into()))
}

fn handle_metrics_api_request() -> Result<(String, String), MetricsError> {
    let device_path = get_device_path().ok_or(MetricsError::BadPath)?;
    let mut device = setup_device(&device_path).ok_or(MetricsError::BadDevice)?;

    const METRICS_COMMAND : &[u8] = b"metrics\n";
    write_to_device(&mut device, METRICS_COMMAND).map_err(|_| MetricsError::WriteError)?;

    let raw_data = read_from_device(&mut device).map_err(|_| MetricsError::ReadError)?;
    if raw_data.is_empty() { return Err(MetricsError::ReadError); }

    let message = String::from_utf8_lossy(&raw_data);
    try_parse_metrics(&message).ok_or(MetricsError::BadMetrics)
}

fn respond_to_api_request(file_name: String) -> Response {
    match file_name.as_str() {
        "server-version" => {
            Response::new(std::env!("CARGO_PKG_VERSION").into())
        },
        "metrics" => {
            let body = match handle_metrics_api_request() {
                Ok((path, time)) =>
                    format!(r#"{{ "error": "none", "path": "{}", "time": "{}" }}"#, path, time),
                Err(error) =>
                    format!(r#"{{ "error": "{}", "path": "", "time": "" }}"#, error),
            };
            Response::new(body.into())
        },
        _ => {
            let mut response = Response::new("404 Not found".into());
            let status = response.status_mut();
            *status = StatusCode::NOT_FOUND;
            response
        },
    }
}

fn write_to_device(serial: &mut SystemPort, buffer: &[u8]) -> std::io::Result<()> {
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

fn read_from_device(serial: &mut SystemPort) -> std::io::Result<Vec<u8>> {
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

pub fn setup_device(path: &str) -> Option<SystemPort> {
    use serial::*;

    let path = PathBuf::from(path);
    let mut device = SystemPort::open(&path).ok()?;
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
    if let Some(p) = get_device_path() {
        println!("Using '{}' as a path to the device.", p);
    } else {
        eprintln!("No device specified. Please provide the path to the device as an argument.");
        return;
    }

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
        .or(not_found);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8000))
        .await;
}
