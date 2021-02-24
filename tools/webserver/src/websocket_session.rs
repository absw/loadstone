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
        let final_response = match result {
            Some(()) => Self::DONE,
            None => Self::FAIL,
        };
        self.send_response(final_response).await?;
        result
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
