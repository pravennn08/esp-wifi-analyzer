//! All board peripheral selection and startup sequencing lives here.
use esp_hal::{clock::CpuClock, peripherals::WIFI, timer::timg::TimerGroup};

pub fn init() -> WIFI<'static> {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    // Same reclaimed RAM budget as esp-generate 2.0.0's ESP32 Wi-Fi template.
    // Measure available heap under real scan load before changing this budget.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    let timers = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timers.timer0, peripherals.FROM_CPU_INTR0);
    peripherals.WIFI
}
