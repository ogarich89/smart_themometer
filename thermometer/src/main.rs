use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::Mutex,
    time,
};

#[derive(Default)]
struct Temperature(Mutex<f32>);

impl Temperature {
    pub async fn get(&self) -> f32 {
        *self.0.lock().await
    }

    pub async fn set(&self, val: f32) {
        *self.0.lock().await = val
    }
}

struct SmartThermometer {
    temperature: Arc<Temperature>,
    connected: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
}

impl SmartThermometer {
    async fn new(address: impl ToSocketAddrs) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(address).await?;
        let timeout = Duration::from_secs(2);

        let connected = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let temperature = Arc::new(Temperature::default());

        let connected_clone = connected.clone();
        let finished_clone = finished.clone();
        let temperature_clone = temperature.clone();

        tokio::spawn(async move {
            loop {
                if finished_clone.load(Ordering::SeqCst) {
                    return;
                }

                let mut buf = [0; 4];

                if let Err(err) = time::timeout(timeout, socket.recv_from(&mut buf)).await {
                    connected_clone.store(false, Ordering::SeqCst);
                    println!("Can't receive datagram: {}", err);
                } else {
                    connected_clone.store(true, Ordering::SeqCst);
                }
                temperature_clone.set(f32::from_be_bytes(buf)).await;
            }
        });

        Ok(Self {
            temperature,
            connected,
            finished,
        })
    }
    pub async fn get_temperature(&self) -> f32 {
        self.temperature.get().await
    }
}

impl Drop for SmartThermometer {
    fn drop(&mut self) {
        self.finished.store(false, Ordering::SeqCst)
    }
}
#[tokio::main]
async fn main() {
    let thermometer = SmartThermometer::new("127.0.0.1:3000").await.unwrap();
    loop {
        time::sleep(Duration::from_secs(2)).await;
        let temperature = thermometer.get_temperature().await;
        if thermometer.connected.load(Ordering::SeqCst) {
            println!("The temperature is {}", temperature);
        }
    }
}
