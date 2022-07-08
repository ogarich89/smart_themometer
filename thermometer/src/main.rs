use std::{
    error::Error,
    net::{ToSocketAddrs, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Default)]
struct Temperature(Mutex<f32>);

impl Temperature {
    pub fn get(&self) -> f32 {
        *self.0.lock().unwrap()
    }

    pub fn set(&self, val: f32) {
        *self.0.lock().unwrap() = val
    }
}

struct SmartThermometer {
    temperature: Arc<Temperature>,
    connected: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
}

impl SmartThermometer {
    fn new(address: impl ToSocketAddrs) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(address)?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;

        let connected = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let temperature = Arc::new(Temperature::default());

        let connected_clone = connected.clone();
        let finished_clone = finished.clone();
        let temperature_clone = temperature.clone();

        thread::spawn(move || loop {
            if finished_clone.load(Ordering::SeqCst) {
                return;
            }

            let mut buf = [0; 4];
            if let Err(err) = socket.recv_from(&mut buf) {
                connected_clone.store(false, Ordering::SeqCst);
                println!("Can't receive datagram: {}", err);
            } else {
                connected_clone.store(true, Ordering::SeqCst);
            }
            temperature_clone.set(f32::from_be_bytes(buf));
        });

        Ok(Self {
            temperature,
            connected,
            finished,
        })
    }
    pub fn get_temperature(&self) -> f32 {
        self.temperature.get()
    }
}

impl Drop for SmartThermometer {
    fn drop(&mut self) {
        self.finished.store(false, Ordering::SeqCst)
    }
}

fn main() {
    let thermometer = SmartThermometer::new("127.0.0.1:3000").unwrap();
    loop {
        thread::sleep(Duration::from_secs(2));
        let temperature = thermometer.get_temperature();
        if thermometer.connected.load(Ordering::SeqCst) {
            println!("The temperature is {}", temperature);
        }
    }
}
