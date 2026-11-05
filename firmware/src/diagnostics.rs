//! Only the console task writes ordinary UART output; panic output is exceptional.
pub const PROMPT: &[u8] = b"wifi> ";
pub const INTRO: &str = concat!(
    "\r\nESP32 Wi-Fi Analyzer ",
    env!("CARGO_PKG_VERSION"),
    "\r\n",
    "Target: ESP32 / UART0 at 115200 baud\r\n",
    "Type 'help' for commands.\r\n",
);
