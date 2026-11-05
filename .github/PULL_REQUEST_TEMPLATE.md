## Summary

Describe the problem and the resulting behavior. Link the related issue, if any.

## Changes

-

## Validation

List the checks you actually ran and their results. Mark checks not run as such;
do not treat a successful build as hardware validation.

| Check | Result |
|---|---|
| Formatting, host tests, and Clippy | |
| ESP32 build and firmware Clippy, if applicable | |
| Hardware checks, if applicable | |

For board tests, include the board/chip revision, commands, duration, and observed
results. Redact private network names, identifiers, and credentials from logs.

## Impact and limitations

Describe command or compatibility changes, buffer/RAM/flash effects, and any
behavior that still needs testing. For a documentation-only change, state that
here and use the relevant link/YAML checks instead of firmware tests.

## Review checklist

- [ ] The change is focused and follows CONTRIBUTING.md.
- [ ] Relevant regression tests and documentation were updated, or are not needed.
- [ ] User-facing changes have an Unreleased changelog entry, or do not need one.
- [ ] Third-party notices are preserved; logs and files contain no credentials.

Do not disclose vulnerability details in a public pull request. Follow SECURITY.md.
