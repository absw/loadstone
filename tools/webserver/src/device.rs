use std::path::PathBuf;

pub use serial::SystemPort;

pub fn new_system_port(path: &str) -> Option<SystemPort> {
    use serial::*;

    let path = PathBuf::from(path);
    let mut port = SystemPort::open(&path).ok()?;
    port.reconfigure(&|s| {
        s.set_baud_rate(Baud115200)?;
        s.set_char_size(Bits8);
        s.set_parity(ParityNone);
        s.set_stop_bits(Stop1);
        s.set_flow_control(FlowNone);
        Ok(())
    }).ok()?;

    Some(port)
}

pub fn write_to_device(serial: &mut SystemPort, buffer: &[u8]) -> std::io::Result<()> {
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

pub fn read_from_device(serial: &mut SystemPort) -> std::io::Result<Vec<u8>> {
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
