# Security Policy

## Reporting a Vulnerability

Do NOT open a public GitHub issue for security vulnerabilities.

Report privately with a description of the vulnerability, steps to reproduce, and potential impact. We respond within 48 hours.

## Supported Versions

| Version | Supported |
|---|---|
| 0.1.x | ✅ Yes |

## Known Considerations

- This contract is pre-audit. Do not use with significant real funds until a full audit is completed.
- Rounding: basis point division may result in dust amounts not distributed. The contract intentionally skips zero-amount transfers.
- Auth: only the split owner can update, deactivate, or reactivate their split.

## Scope

- `src/lib.rs` — all contract functions
- Auth bypass or unauthorized split modification
- Integer overflow in share calculations
- Storage manipulation
