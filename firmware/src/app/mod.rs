//! Composition and task ownership. Domain state stays local to its owner.
pub mod messages;
mod tasks;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;

use crate::platform::{radio::Radio, serial};

pub async fn run(spawner: Spawner, peripheral: WIFI<'static>) -> ! {
    match Radio::new(peripheral) {
        Ok(radio) => {
            serial::boot("Radio initialized. Starting application task.");
            spawner.spawn(tasks::analyzer(radio).expect("allocate analyzer task"));
        }
        Err(error) => {
            esp_println::println!("Radio initialization failed: {:?}", error);
            // Stay observable instead of entering a boot-reset loop.
        }
    }
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
