// SPDX-License-Identifier: Apache-2.0

//! The console task exclusively owns both halves of UART0 and all ordinary output.
use analyzer_core::{
    cli::{
        Command, InputEvent, LineEditor, LineError,
        report::write_status,
        terminal::{AnsiColor, TerminalSettings, write_colored},
    },
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
    diagnostics::{COLOR_PROMPT, INTRO, PROMPT},
};

struct Output {
    uart: UartTx<'static, Async>,
    settings: TerminalSettings,
    editable_columns: Option<usize>,
}

impl Output {
    async fn write(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            // The pinned HAL's TxError is uninhabited and nonempty writes progress.
            let written = self
                .uart
                .write_async(bytes)
                .await
                .expect("UART0 transmit failed");
            bytes = &bytes[written..];
        }
    }

    async fn prompt(&mut self, line: &[u8]) {
        self.write(if self.settings.color {
            COLOR_PROMPT
        } else {
            PROMPT
        })
        .await;
        self.write(line).await;
        self.editable_columns = Some(PROMPT.len() + line.len());
    }

    async fn finish_input(&mut self) {
        self.editable_columns = None;
        self.write(b"\r\n").await;
    }

    async fn clear_input(&mut self) {
        const BLANKS: [u8; PROMPT.len() + CLI_LINE_BYTES] = [b' '; PROMPT.len() + CLI_LINE_BYTES];
        if let Some(columns) = self.editable_columns.take() {
            if columns >= self.settings.width {
                // A long edit may have wrapped. Leave it intact and redraw on a new row.
                self.write(b"\r\n").await;
            } else {
                self.write(b"\r").await;
                self.write(&BLANKS[..columns]).await;
                self.write(b"\r").await;
            }
        }
    }

    async fn message(&mut self, bytes: &[u8]) {
        if self.settings.color {
            for (prefix, color) in [
                (b"Error:".as_slice(), b"\x1b[31m"),
                (b"Warning:", b"\x1b[33m"),
            ] {
                if let Some(rest) = bytes.strip_prefix(prefix) {
                    self.write(color).await;
                    self.write(prefix).await;
                    self.write(b"\x1b[0m").await;
                    self.write(rest).await;
                    return;
                }
            }
        }
        self.write(bytes).await;
    }
}

pub async fn run(uart: Uart<'static, Async>) -> ! {
    let (mut rx, tx) = uart.split();
    let mut out = Output {
        uart: tx,
        settings: TerminalSettings::default(),
        editable_columns: None,
    };
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
                    out.clear_input().await;
                }
                match reply.kind {
                    ReplyKind::Progress(text) => {
                        out.write(text.as_bytes()).await;
                        report_active = false;
                        out.prompt(editor.line()).await;
                        continue;
                    }
                    ReplyKind::Text(text) => out.message(text.as_bytes()).await,
                    ReplyKind::Status(status) => {
                        report.clear();
                        let formatted = write_colored(
                            &mut report,
                            out.settings.color,
                            AnsiColor::Cyan,
                            "Device status",
                        )
                        .and_then(|()| report.write_str("\r\n"))
                        .and_then(|()| {
                            write_status(&mut report, env!("CARGO_PKG_VERSION"), &status)
                        })
                        .and_then(|()| write!(report, "UART receive errors: {rx_errors}\r\n"));
                        if formatted.is_ok() {
                            out.write(report.as_bytes()).await;
                        } else {
                            out.message(b"Error: status report exceeded its buffer.\r\n")
                                .await;
                        }
                    }
                    ReplyKind::Terminal(settings) => {
                        out.settings = settings;
                        report.clear();
                        let formatted = write_colored(
                            &mut report,
                            settings.color,
                            AnsiColor::Cyan,
                            "Terminal settings",
                        )
                        .and_then(|()| {
                            write!(
                                report,
                                "\r\nColor: {}\r\nWidth: {} columns\r\nLayout: {}\r\n",
                                if settings.color { "on" } else { "off" },
                                settings.width,
                                settings.layout.as_str()
                            )
                        });
                        if formatted.is_ok() {
                            out.write(report.as_bytes()).await;
                        } else {
                            out.message(
                                b"Error: terminal settings report exceeded its buffer.\r\n",
                            )
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
                out.finish_input().await;
                out.message(b"Error: UART receive error. Press Enter to discard the line.\r\n")
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
                        if let Some(columns) = out.editable_columns.as_mut() {
                            *columns += 1;
                        }
                    }
                }
                InputEvent::Erase => {
                    if !report_active {
                        out.write(b"\x08 \x08").await;
                        if let Some(columns) = out.editable_columns.as_mut() {
                            *columns = columns.saturating_sub(1);
                        }
                    }
                }
                event => {
                    if !report_active {
                        out.finish_input().await;
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
                                out.message(b"Error: request queue busy. Try again.\r\n")
                                    .await;
                            } else {
                                let _ = pending.push(id);
                                next_request_id = next_request_id.wrapping_add(1);
                            }
                        }
                        InputEvent::Rejected(error) => out.message(error_text(error)).await,
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
