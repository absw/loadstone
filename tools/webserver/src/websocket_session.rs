use warp::ws::{WebSocket, Message};
use futures::{
    prelude::*,
    stream::{SplitSink, SplitStream}
};
use serial::SystemPort;
use crate::xmodem::XModemSession;

pub struct WebSocketSession {
    reciever: SplitStream<WebSocket>,
    sender: SplitSink<WebSocket, Message>,
    device: Option<SystemPort>,
}

impl WebSocketSession {
    const NEXT : u8 = 0x11;
    const FAIL : u8 = 0x22;
    const DONE : u8 = 0x33;

    pub async fn run_new(socket: WebSocket, device: SystemPort) -> Option<()> {
        Self::new(socket, device).run().await
    }

    fn new(socket: WebSocket, device: SystemPort) -> Self {
        let (sender, reciever) = socket.split();
        Self {
            sender,
            reciever,
            device: Some(device),
        }
    }

    async fn run(mut self) -> Option<()> {
        let result = self.run_inner().await;
        if result.is_none() {
            self.send_response(Self::FAIL).await?;
            return None;
        }

        self.send_response(Self::DONE).await?;
        self.validate_transfer().await?;

        result
    }

    fn contains_subslice(slice: &[u8], subslice: &[u8]) -> bool {
        slice.windows(subslice.len())
            .any(|w| w == subslice)
    }

    async fn validate_transfer(&mut self) -> Option<()> {
        use std::io::Read;
        use std::time::*;

        let timeout = Instant::now() + Duration::from_secs(30);
        let mut interval = tokio::time::interval(Duration::from_millis(250));

        let mut device = self.device.take().unwrap();
        let mut buffer = Vec::<u8>::new();

        while Instant::now() < timeout {
            let mut append_buffer = Vec::<u8>::new();
            let result = device.read_to_end(&mut append_buffer);
            buffer.extend(append_buffer);

            if let Err(e) = result {
                let is_nonfatal_error = (e.kind() == std::io::ErrorKind::Interrupted)
                    || (e.kind() == std::io::ErrorKind::TimedOut);
                if !is_nonfatal_error { return None; }
            }

            const SUCCESS_MESSAGE : &[u8] = b"Image transfer complete!";
            if Self::contains_subslice(&buffer, SUCCESS_MESSAGE) { return Some(()) }

            interval.tick().await;
        }

        None
    }

    async fn run_inner(&mut self) -> Option<()> {
        println!("Starting upload...");
        let packet_count = self.get_first_packet().await?;

        let mut xmodem = XModemSession::new(self.device.take().unwrap())?;
        println!("Started XModem session.");

        for _ in 0..packet_count {
            self.send_response(Self::NEXT).await?;
            let packet = self.get_next_packet().await?;
            xmodem.send(&packet)?;
        }

        self.device = Some(xmodem.return_port());

        Some(())
    }

    async fn send_response(&mut self, content: u8) -> Option<()> {
        let response = warp::ws::Message::binary(vec!(content));
        self.sender.send(response).await.ok()
    }

    async fn get_next_packet(&mut self) -> Option<Vec<u8>> {
        let packet = self.reciever.next().await?;
        let message = packet.ok()?;
        Some(message.into_bytes())
    }

    async fn get_first_packet(&mut self) -> Option<u32> {
        let packet = self.reciever.next().await?;
        let message = packet.ok()?;
        let bytes = message.as_bytes();
        if bytes.len() == 4 {
            let mut data = [0u8; 4];
            data.clone_from_slice(bytes);
            Some(u32::from_be_bytes(data))
        } else {
            None
        }
    }
}
