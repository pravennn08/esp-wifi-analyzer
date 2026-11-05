//! Bootstrap output only. The interactive UART0 RX/TX task is the next milestone.
//! Route ordinary output through that task once it exists, to avoid mixed output.
pub fn boot(message: &str) {
    esp_println::println!("{}", message);
}
