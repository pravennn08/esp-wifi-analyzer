use analyzer_core::{
    cli::{
        Command, InputEvent, LineEditor, LineError,
        report::{HELP_TEXT, write_status},
    },
    config::{CLI_LINE_BYTES, STATUS_REPORT_BYTES},
    error::CommandError,
    state::{RadioState, ScannerState, StatusSnapshot},
};

fn submit<const N: usize>(editor: &mut LineEditor<N>, input: &[u8]) -> Vec<InputEvent> {
    input
        .iter()
        .map(|byte| editor.push(*byte))
        .filter(|event| {
            !matches!(
                event,
                InputEvent::None | InputEvent::Echo(_) | InputEvent::Erase
            )
        })
        .collect()
}

#[test]
fn crlf_submits_once_and_lf_only_also_works() {
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    assert_eq!(
        submit(&mut editor, b"help\r\nstatus\n"),
        vec![
            InputEvent::Command(Command::Help),
            InputEvent::Command(Command::Status),
        ]
    );
}

#[test]
fn backspace_and_delete_edit_the_command_before_parsing() {
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    assert_eq!(
        submit(&mut editor, b"helx\x08p\rstatux\x7fs\n"),
        vec![
            InputEvent::Command(Command::Help),
            InputEvent::Command(Command::Status),
        ]
    );
}

#[test]
fn overflowing_line_never_executes_its_valid_command_prefix() {
    let mut editor = LineEditor::<4>::default();
    assert_eq!(
        submit(&mut editor, b"helpX\x08\nhelp\n"),
        vec![
            InputEvent::Rejected(LineError::TooLong),
            InputEvent::Command(Command::Help),
        ]
    );
}

#[test]
fn terminal_escape_and_non_ascii_input_are_not_executed_or_echoed() {
    for bad in [b'\x1b', 0xff, 0x9b, 0] {
        let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
        submit(&mut editor, b"status");
        assert_eq!(editor.push(bad), InputEvent::None);
        assert!(editor.line().is_empty());
        assert_eq!(
            submit(&mut editor, b"\nhelp\n"),
            vec![
                InputEvent::Rejected(LineError::UnsupportedCharacter),
                InputEvent::Command(Command::Help),
            ]
        );
    }
}

#[test]
fn cancel_recovers_even_after_overflow() {
    let mut editor = LineEditor::<4>::default();
    assert_eq!(
        submit(&mut editor, b"longcommand\x03help\nxxx\x15help\n"),
        vec![
            InputEvent::Cancelled,
            InputEvent::Command(Command::Help),
            InputEvent::Cancelled,
            InputEvent::Command(Command::Help),
        ]
    );
}

#[test]
fn receive_error_discards_input_until_newline() {
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    submit(&mut editor, b"sta");
    editor.discard(LineError::Receive);
    assert_eq!(
        submit(&mut editor, b"tus\nstatus\n"),
        vec![
            InputEvent::Rejected(LineError::Receive),
            InputEvent::Command(Command::Status),
        ]
    );
}

#[test]
fn blank_lines_and_invalid_arguments_are_distinct() {
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    assert_eq!(
        submit(&mut editor, b" \t\nstatus extra\nunknown\n"),
        vec![
            InputEvent::Empty,
            InputEvent::Rejected(LineError::Syntax(CommandError::InvalidArguments)),
            InputEvent::Rejected(LineError::Syntax(CommandError::UnknownCommand)),
        ]
    );
}

#[test]
fn receive_error_after_carriage_return_recovers_at_the_next_newline() {
    let mut editor = LineEditor::<CLI_LINE_BYTES>::default();
    assert_eq!(
        submit(&mut editor, b"help\r"),
        vec![InputEvent::Command(Command::Help)]
    );
    editor.discard(LineError::Receive);
    assert_eq!(
        submit(&mut editor, b"\nstatus\n"),
        vec![
            InputEvent::Rejected(LineError::Receive),
            InputEvent::Command(Command::Status),
        ]
    );
}

#[test]
fn status_report_fits_its_budget_at_maximum_counter_values() {
    let mut out = heapless::String::<STATUS_REPORT_BYTES>::new();
    let mut status = StatusSnapshot {
        uptime_ms: u64::MAX,
        radio: RadioState::Initialized,
        scanner: ScannerState::Idle,
        successful_scans: u32::MAX,
        failed_scans: u32::MAX,
        cached_aps: usize::MAX,
        last_scan_duration_ms: Some(u32::MAX),
        heap_total_bytes: usize::MAX,
        heap_used_bytes: 0,
    };
    write_status(&mut out, "0.1.0", &status).unwrap();
    assert!(out.contains("Radio: initialized\r\n"));
    assert!(out.contains("Scanner: idle\r\n"));
    assert!(out.contains("Scan execution: not implemented"));
    status.radio = RadioState::Failed;
    status.scanner = ScannerState::Faulted;
    status.last_scan_duration_ms = None;
    out.clear();
    write_status(&mut out, "0.1.0", &status).unwrap();
    assert!(out.contains("Radio: initialization failed"));
    assert!(out.contains("Scanner: faulted"));
    assert!(out.contains("Last scan duration: none"));
    assert!(HELP_TEXT.contains("help"));
    assert!(HELP_TEXT.contains("status"));
}
