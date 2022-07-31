use std::{net::SocketAddr, thread, time::Duration};

use tokio::{net::UdpSocket, time::Instant};

struct TemperatureGenerator {
    started: Instant,
}

impl Default for TemperatureGenerator {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl TemperatureGenerator {
    pub fn generate(&self) -> f32 {
        let delay = Instant::now() - self.started;
        24.0 + (delay.as_secs_f32() / 4.0).sin()
    }
}
#[tokio::main]
async fn main() {
    let receiver = "127.0.0.1:3000";

    println!("Receiver address from args: {receiver}");

    let receiver = receiver
        .parse::<SocketAddr>()
        .expect("Valid socket address expected");

    let bind_addr = "127.0.0.1:3001";
    let socket = UdpSocket::bind(bind_addr).await.expect("Can't bind socket");
    let temperature_generator = TemperatureGenerator::default();

    println!("Starting send temperature from {bind_addr} to {receiver}");
    loop {
        let temperature = temperature_generator.generate();
        let bytes = temperature.to_be_bytes();
        let send_result = socket.send_to(&bytes, receiver).await;
        if let Err(err) = send_result {
            println!("Can't send temperature: {}", err)
        }
        thread::sleep(Duration::from_secs(1))
    }
}
