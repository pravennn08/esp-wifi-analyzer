// SPDX-License-Identifier: Apache-2.0

//! The console task exclusively owns both halves of UART0 and all ordinary output.
use analyzer_core::{
    cli::{Command, InputEvent, LineEditor, LineError, report::write_status},
    config::{CLI_LINE_BYTES, REQUEST_QUEUE_DEPTH, STATUS_REPORT_BYTES},
    error::CommandError,
};
use core::fmt::Write;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_hal::{
    Async,
    uart::{Uart, UartTx},
};
use heapless::{String, Vec};

use crate::{
    app::messages::{REPLIES, REQUESTS, ReplyKind, Request, STOP},
    diagnostics::{INTRO, PROMPT},
};

struct Output(UartTx<'static, Async>);

impl Output {
    async fn write(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            // The pinned HAL's TxError is uninhabited and nonempty writes progress.
            let written = self
                .0
                .write_async(bytes)
                .await
                .expect("UART0 transmit failed");
            bytes = &bytes[written..];
        }
    }

    async fn prompt(&mut self, line: &[u8]) {
        self.write(PROMPT).await;
        self.write(line).await;
    }
}

pub async fn run(uart: Uart<'static, Async>) -> ! {
    let (mut rx, tx) = uart.split();
    let mut out = Output(tx);
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    let mut input = [0u8; 1];
    let mut next_request_id = 0u32;
    let mut pending = Vec::<u32, REQUEST_QUEUE_DEPTH>::new();
    let mut report_active = false;
    let mut rx_errors = 0u32;
    let mut report = String::<STATUS_REPORT_BYTES>::new();

    out.write(INTRO.as_bytes()).await;
    out.prompt(&[]).await;

    loop {
        // Both receive futures are cancellation-safe. Replies get priority so
        // continuously pasted input cannot starve the application response.
        match select(REPLIES.receive(), rx.read_async(&mut input)).await {
            Either::First(reply) => {
                if reply.request_id.is_some_and(|id| !pending.contains(&id)) {
                    continue;
                }
                if !report_active {
                    // Hide the editable line once, not between streamed report rows.
                    out.write(b"\r\x1b[2K").await;
                }
                match reply.kind {
                    ReplyKind::Progress(text) => {
                        out.write(text.as_bytes()).await;
                        report_active = false;
                        out.prompt(editor.line()).await;
                        continue;
                    }
                    ReplyKind::Text(text) => out.write(text.as_bytes()).await,
                    ReplyKind::Status(status) => {
                        report.clear();
                        let formatted =
                            write_status(&mut report, env!("CARGO_PKG_VERSION"), &status).and_then(
                                |()| write!(report, "UART receive errors: {rx_errors}\r\n"),
                            );
                        if formatted.is_ok() {
                            out.write(report.as_bytes()).await;
                        } else {
                            out.write(b"Error: status report exceeded its buffer.\r\n")
                                .await;
                        }
                    }
                }
                report_active = !reply.complete;
                if reply.complete {
                    if let Some(id) = reply.request_id
                        && let Some(index) = pending.iter().position(|pending| *pending == id)
                    {
                        pending.swap_remove(index);
                    }
                    out.prompt(editor.line()).await;
                }
            }
            Either::Second(Err(_)) => {
                rx_errors = rx_errors.saturating_add(1);
                editor.discard(LineError::Receive);
                out.write(b"\r\nError: UART receive error. Press Enter to discard the line.\r\n")
                    .await;
                // Avoid a tight loop if persistent electrical noise causes errors.
                Timer::after(Duration::from_millis(10)).await;
            }
            Either::Second(Ok(0)) => {}
            Either::Second(Ok(_)) => match editor.push(input[0]) {
                InputEvent::None => {}
                InputEvent::Echo(byte) => {
                    if !report_active {
                        out.write(&[byte]).await;
                    }
                }
                InputEvent::Erase => {
                    if !report_active {
                        out.write(b"\x08 \x08").await;
                    }
                }
                event => {
                    if !report_active {
                        out.write(b"\r\n").await;
                    }
                    match event {
                        InputEvent::StopRequested | InputEvent::Command(Command::Stop) => {
                            // An urgent stop remains available even if requests are full.
                            STOP.signal(next_request_id);
                        }
                        InputEvent::Command(command) => {
                            let id = next_request_id;
                            if pending.is_full()
                                || REQUESTS.try_send(Request { id, command }).is_err()
                            {
                                out.write(b"Error: request queue busy. Try again.\r\n")
                                    .await;
                            } else {
                                let _ = pending.push(id);
                                next_request_id = next_request_id.wrapping_add(1);
                            }
                        }
                        InputEvent::Rejected(error) => out.write(error_text(error)).await,
                        InputEvent::Cancelled => out.write(b"Line cleared.\r\n").await,
                        _ => {}
                    }
                    if !report_active {
                        out.prompt(&[]).await;
                    }
                }
            },
        }
    }
}

fn error_text(error: LineError) -> &'static [u8] {
    match error {
        LineError::TooLong => b"Error: command too long; entire line discarded.\r\n",
        LineError::UnsupportedCharacter => b"Error: unsupported character; use ASCII commands without terminal escape sequences.\r\n",
        LineError::Receive => b"UART input recovered.\r\n",
        LineError::Syntax(CommandError::UnknownCommand) => b"Error: unknown command. Type help.\r\n",
        LineError::Syntax(_) => b"Error: invalid arguments. Type help.\r\n",
    }
}
