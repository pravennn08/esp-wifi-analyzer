#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandError {
    Empty,
    UnknownCommand,
    InvalidArguments,
    InvalidChannel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanError {
    Busy,
    UnsupportedChannel,
    Driver,
    RecoveryRequired,
}
