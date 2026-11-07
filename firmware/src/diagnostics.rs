// SPDX-License-Identifier: Apache-2.0

//! Only the console task writes ordinary UART output; panic output is exceptional.
pub const PROMPT: &[u8] = b"esp-wifi> ";
pub const COLOR_PROMPT: &[u8] = b"\x1b[36mesp-wifi\x1b[32m>\x1b[0m ";
pub const INTRO: &str = concat!(
    "\r\nESP32 Wi-Fi Analyzer ",
    env!("CARGO_PKG_VERSION"),
    "\r\n",
    "Target: ESP32 / UART0 at 115200 baud\r\n",
    "Type 'help' for commands.\r\n",
);
