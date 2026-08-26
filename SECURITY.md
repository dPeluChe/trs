# Security Policy

## Supported versions

Only the latest release receives fixes. Run `trs upgrade` to stay current.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting
(Security → Report a vulnerability) on this repository.
You should get a first response within 72 hours.

Please do not open public issues for security reports.

## Scope notes

trs executes the commands an AI agent (or you) already intended to run,
it does not grant new execution capability. Reports we especially care
about: credential leakage through compacted output (trs deliberately
preserves and redacts credential-bearing lines, see `redact_secrets`),
hook-template injection, and anything in `install.sh` / the release
pipeline.
