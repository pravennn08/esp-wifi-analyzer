use super::Command;
use crate::{error::CommandError, model::ScanOptions};

pub fn parse(line: &str) -> Result<Command, CommandError> {
    let mut tokens = line.split_ascii_whitespace();
    let command = tokens.next().ok_or(CommandError::Empty)?;
    match command {
        "help" | "status" => {
            if tokens.next().is_some() {
                return Err(CommandError::InvalidArguments);
            }
            Ok(if command == "help" {
                Command::Help
            } else {
                Command::Status
            })
        }
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
                let channel: u8 = tokens
                    .next()
                    .ok_or(CommandError::InvalidArguments)?
                    .parse()
                    .map_err(|_| CommandError::InvalidChannel)?;
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
