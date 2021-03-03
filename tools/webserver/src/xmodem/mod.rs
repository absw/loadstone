mod packet;
use packet::*;

use std::{
    io::{Read, Write},
    thread::sleep,
    time::{Instant, Duration},
};

use serial::SystemPort;

pub struct XModemSession {
    port: SystemPort,
    block_number: u8,
}

impl XModemSession {
    pub fn return_port(mut self) -> SystemPort {
        self.send_terminal_packet();

        // Swaps out the system port with empty space, and then forgets self.
        // This means we can get port out from self. We can forget self because
        // everything that needs dropping has been moved out.
        use std::mem::{replace, forget, MaybeUninit};
        let port = replace(
            &mut self.port,
            unsafe { MaybeUninit::<SystemPort>::uninit().assume_init() }
        );
        forget(self);

        port
    }

    pub fn new(mut port: SystemPort) -> Option<Self> {
        let write_success = port.write_all( b"flash bank=2\n").is_ok();
        let mut discarded_data = Vec::new();
        let _ = port.read_to_end(&mut discarded_data);

        if write_success {
            let mut s = Self {
                port,
                block_number: 0,
            };
            s.wait_for_negative_acknowledge();
            Some(s)
        } else {
            None
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Option<()> {
        self.block_number = self.block_number.wrapping_add(1);
        let packet = Packet::new(self.block_number, &data);
        self.try_write_packet(packet)
    }

    fn try_write_packet(&mut self, packet: Packet) -> Option<()> {
        const MAX_ATTEMPTS : usize = 10;
        for _ in 0..MAX_ATTEMPTS {
            self.write_packet(&packet)?;
            let acknowledged = self.wait_for_response()?;
            if acknowledged { return Some(()); }
        }
        None
    }

    fn write_packet(&mut self, packet: &Packet) -> Option<()> {
        self.port.write_all(packet.data()).ok().map(|_| ())
    }

    fn read(&mut self) -> Option<bool> {
        const ACKNOWLEDGE : u8 = 0x06;
        const NEGATIVE_ACKNOWLEDGE : u8 = 0x15;
        let mut read_buffer = [0u8; 1];
        self.port.read_exact(&mut read_buffer).ok()?;
        match read_buffer[0] {
            ACKNOWLEDGE => Some(true),
            NEGATIVE_ACKNOWLEDGE => Some(false),
            _ => None,
        }
    }

    fn wait_for_response(&mut self) -> Option<bool> {
        const TIMEOUT : Duration = Duration::from_secs(10);
        const DELAY : Duration = Duration::from_millis(500);
        let timeout_point = Instant::now() + TIMEOUT;

        while Instant::now() < timeout_point {
            if let Some(r) = self.read() {
                return Some(r);
            }
            sleep(DELAY);
        }

        None
    }

    fn wait_for_negative_acknowledge(&mut self) -> Option<()> {
        match self.wait_for_response()? {
            false => Some(()),
            true => None,
        }
    }

    fn send_terminal_packet(&mut self) {
        self.block_number = self.block_number.wrapping_add(1);
        self.write_packet(&Packet::Terminal);
    }
}

impl Drop for XModemSession {
    fn drop(&mut self) {
        self.send_terminal_packet();
    }
}
