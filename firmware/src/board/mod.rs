// SPDX-License-Identifier: Apache-2.0

//! All board peripheral selection and startup sequencing lives here.
use esp_hal::{
    Async,
    clock::CpuClock,
    peripherals::WIFI,
    timer::timg::TimerGroup,
    uart::{Config, Uart},
};

pub struct Resources {
    pub wifi: WIFI<'static>,
    pub console: Uart<'static, Async>,
}

pub fn init() -> Resources {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    // Same reclaimed RAM budget as esp-generate 2.0.0's ESP32 Wi-Fi template.
    // Measure available heap under real scan load before changing this budget.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    let timers = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timers.timer0, peripherals.FROM_CPU_INTR0);

    // Classic ESP32 dev boards route the USB bridge to UART0: RX=3, TX=1.
    let console = Uart::new(peripherals.UART0, Config::default().with_baudrate(115_200))
        .expect("valid UART0 configuration")
        .with_rx(peripherals.GPIO3)
        .with_tx(peripherals.GPIO1)
        .into_async();

    Resources {
        wifi: peripherals.WIFI,
        console,
    }
}
