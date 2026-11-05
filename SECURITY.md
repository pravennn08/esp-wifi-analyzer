# Security Policy

## Supported versions

ESP32 Wi-Fi Analyzer is under active development. Security fixes are maintained
on the latest commit of `main` on a best-effort basis.

| Build | Security maintenance |
|---|---|
| Latest `main` | Best-effort investigation and fixes |
| Earlier commits | Update to the latest `main`; separate backports are not maintained |
| Forks and modified builds | Report to their maintainer first; identify any issue that also affects upstream |

The firmware currently displays `0.1.0`. Include the exact commit hash in reports;
the displayed version does not uniquely identify a build. This policy does not
imply a security audit, certification, or a guaranteed support period.

## Reporting a vulnerability

Do not publish exploit details, proof-of-concept payloads, credentials, or
unredacted network data in a public issue, pull request, or discussion.

1. Check the repository's [Security Advisories](https://github.com/pravennn08/esp-wifi-analyzer/security/advisories)
   page. If **Report a vulnerability** is available, use it to send a private
   report to the maintainers.
2. If that option is unavailable, request a private reporting channel from the
   [repository maintainer](https://github.com/pravennn08), using a private contact
   method published on their profile if one is available.
3. If no private contact method is published, open an issue titled
   `Security contact request`. Include only a request for a private reporting
   channel. Do not describe the vulnerability or attach evidence in that issue;
   wait for a private channel before sharing details.

The project has not published a dedicated security email address. Do not assume
that a public bug-report form or a contact-request issue is confidential.
GitHub explains the private-reporting workflow and its fallback in
[Privately reporting a security vulnerability](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately).

### Information to include privately

- Affected commit or version and whether the issue reproduces on current `main`.
- Board/module, chip revision, flash size, and relevant tool/dependency versions.
- The affected component: CLI input, scan processing, report rendering, task
  scheduling, dependency, build script, or CI workflow.
- A clear description of the impact and the access or conditions required.
- Minimal reproduction steps or a proof of concept using a controlled test setup.
- Relevant redacted logs, crash output, or affected file locations.
- Any known workaround or proposed fix, if available.

Share only the information needed to reproduce the issue. Use synthetic network
names and identifiers when possible; remove passwords, tokens, personal data,
private SSIDs, BSSIDs, and device MAC addresses from supporting material.

## Report handling and disclosure

Maintainers will assess the report, attempt to reproduce it, determine affected
code and dependencies, and coordinate a fix or mitigation with the reporter.
They may request additional information or help validating a candidate fix.

Please coordinate public disclosure so a fix or useful mitigation can be made
available first. Timing depends on severity, exploitability, reproducibility,
and dependencies. This is a volunteer-maintained project with no guaranteed
response time, remediation deadline, or bug-bounty program.

Confirmed fixes should include regression coverage where practical and identify
the affected and fixed commits. Maintainers may publish a GitHub security
advisory or a changelog entry with impact, prerequisites, and upgrade guidance.
Public credit should be agreed with the reporter before publication.

For issues in Espressif's HAL, wireless driver binaries, or another dependency,
maintainers may coordinate with the upstream project. Include the dependency
version and explain how this firmware is affected; an upstream issue can still
require a dependency update or mitigation here.

## Security scope and trust boundaries

- Nearby access points supply untrusted wireless data. Names, metadata, and
  repeated observations must be treated as input, not trusted configuration.
- The serial CLI is a local diagnostic interface without authentication or
  encryption. Someone with access to the USB/serial connection can issue commands
  and read diagnostic observations. Do not expose the serial port through an
  untrusted remote bridge.
- Scan snapshots and RSSI history are held in RAM and are not persisted by the
  application. Serial terminals, host logs, and crash output may retain data
  independently; resetting the board does not delete those host-side records.
- The current application does not provide an HTTP API, web dashboard, remote
  management interface, or OTA update service. Adding one requires a separate
  review of authentication, transport security, input limits, and update trust.
- The project setup does not configure secure boot, flash encryption, or a
  protected update chain. Flashing and physical access are not access-control
  boundaries established by this firmware.
- Rust, `no_std`, bounded application buffers, and an unsafe-free core do not
  eliminate vulnerabilities in HAL integration, dependencies, wireless driver
  binaries, hardware, or scheduling behavior.

Examples worth reporting privately include memory corruption, terminal-control
injection, unintended data disclosure, command parsing that bypasses validation,
or a reproducible attacker-triggered crash or resource exhaustion.

Ordinary scan variability, weak RSSI, an AP missed during a scan, and misleading
channel-utilization expectations are generally functional issues unless there
is a demonstrated security impact. The analyzer reports observations and
advertised security modes; it does not certify a network as secure.

## Testing and applying fixes

Use controlled hardware and networks you own or have permission to test.
Avoid collecting other people's credentials or disrupting unrelated networks.
Do not use production secrets in reproduction cases.

To apply a fix, update to the maintainer-identified fixed commit, review its
dependency and compatibility notes, rebuild, and flash the affected boards.
There is no automatic update service. Keep `Cargo.lock` aligned with the reviewed
source, and do not assume a successful build or CI run proves hardware security.

Use [CONTRIBUTING.md](CONTRIBUTING.md) for ordinary bugs, features, and development
checks. Community-conduct concerns follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
