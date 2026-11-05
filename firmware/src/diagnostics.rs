use crate::platform::serial;

pub fn boot_banner() {
    serial::boot(concat!("ESP32 Wi-Fi Analyzer ", env!("CARGO_PKG_VERSION")));
    serial::boot("Target: ESP32 / 2.4 GHz / UART0 at 115200 baud");
    serial::boot("Foundation build: radio initialization and async runtime.");
    serial::boot("Interactive CLI and scan execution are not implemented yet.");
}
