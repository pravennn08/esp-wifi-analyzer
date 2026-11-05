use super::Command;
use crate::{
    config::{DEFAULT_WATCH_INTERVAL_SECS, MAX_WATCH_COUNT, MAX_WATCH_INTERVAL_SECS},
    error::CommandError,
    model::{ScanOptions, WatchOptions},
};

pub fn parse(line: &str) -> Result<Command, CommandError> {
    let mut tokens = line.split_ascii_whitespace();
    let command = tokens.next().ok_or(CommandError::Empty)?;
    match command {
        "help" | "status" | "networks" | "channels" | "stop" => {
            if tokens.next().is_some() {
                return Err(CommandError::InvalidArguments);
            }
            Ok(match command {
                "help" => Command::Help,
                "status" => Command::Status,
                "networks" => Command::Networks,
                "channels" => Command::Channels,
                _ => Command::Stop,
            })
        }
        "history" => {
            let bssid = tokens
                .next()
                .ok_or(CommandError::InvalidArguments)?
                .parse()
                .map_err(|_| CommandError::InvalidArguments)?;
            if tokens.next().is_some() {
                return Err(CommandError::InvalidArguments);
            }
            Ok(Command::History(bssid))
        }
        "watch" => parse_watch_options(&mut tokens).map(Command::Watch),
        "scan" => {
            let mut options = ScanOptions {
                channel: None,
                passive: false,
            };
            parse_scan_options(&mut tokens, &mut options)?;
            Ok(Command::Scan(options))
        }
        _ => Err(CommandError::UnknownCommand),
    }
}

fn parse_scan_options(
    tokens: &mut core::str::SplitAsciiWhitespace<'_>,
    options: &mut ScanOptions,
) -> Result<(), CommandError> {
    while let Some(option) = tokens.next() {
        match option {
            "--passive" if !options.passive => options.passive = true,
            "--channel" if options.channel.is_none() => {
                let value = tokens.next().ok_or(CommandError::InvalidArguments)?;
                if !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(CommandError::InvalidChannel);
                }
                let channel: u8 = value.parse().map_err(|_| CommandError::InvalidChannel)?;
                // Syntax only. The radio adapter must enforce the configured region.
                if !(1..=14).contains(&channel) {
                    return Err(CommandError::InvalidChannel);
                }
                options.channel = Some(channel);
            }
            _ => return Err(CommandError::InvalidArguments),
        }
    }
    Ok(())
}

fn parse_watch_options(
    tokens: &mut core::str::SplitAsciiWhitespace<'_>,
) -> Result<WatchOptions, CommandError> {
    let mut interval_secs = None;
    let mut count = None;
    while let Some(option) = tokens.next() {
        let value = tokens.next().ok_or(CommandError::InvalidArguments)?;
        match option {
            "--interval" if interval_secs.is_none() => {
                let value = value.strip_suffix('s').unwrap_or(value);
                let seconds = parse_unsigned(value)?;
                if !(1..=MAX_WATCH_INTERVAL_SECS).contains(&seconds) {
                    return Err(CommandError::InvalidArguments);
                }
                interval_secs = Some(seconds);
            }
            "--count" if count.is_none() => {
                let attempts = parse_unsigned(value)?;
                if !(1..=MAX_WATCH_COUNT).contains(&attempts) {
                    return Err(CommandError::InvalidArguments);
                }
                count = Some(attempts);
            }
            _ => return Err(CommandError::InvalidArguments),
        }
    }
    Ok(WatchOptions {
        interval_secs: interval_secs.unwrap_or(DEFAULT_WATCH_INTERVAL_SECS),
        count,
    })
}

fn parse_unsigned(value: &str) -> Result<u32, CommandError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CommandError::InvalidArguments);
    }
    value.parse().map_err(|_| CommandError::InvalidArguments)
}
