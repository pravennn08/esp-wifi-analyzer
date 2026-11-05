#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]

mod app;
mod board;
mod diagnostics;
mod platform;

use embassy_executor::Spawner;
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let resources = board::init();
    app::run(spawner, resources).await
}
