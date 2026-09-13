# ynab-cli

A typed Rust SDK (`ynab-sdk`) and JSON-first CLI for the YNAB v1 API.

## Install

```sh
cargo install --path crates/ynab-cli
```

## Authentication

This release uses a YNAB personal access token. Set `YNAB_ACCESS_TOKEN` in the runtime environment, or store it through `ynab-cli auth store`; tokens are never accepted as command-line arguments or plaintext files.

```sh
ynab-cli plan list
ynab-cli --output json payee create --plan last-used --name Acme
```

Successful commands write exactly one JSON value to stdout by default. Diagnostics use stderr. `--output table` is an interactive rendering of that same data.

## Money inputs

CLI money arguments are decimal currency amounts, converted exactly to integer milliunits. Inputs accept at most three fractional digits: `12`, `12.5`, `12.500`, and `-0.001` are exact; binary floating point is never used.

## Plan selection and writes

Read commands default to YNAB's `last-used` plan and accept `--plan <uuid|last-used|default>`. Every write requires an explicit `--plan`. Destructive, bulk-update, and import commands also require `--yes`; omission is an invocation error before credentials are loaded or HTTP is attempted.

Transaction lists accept one optional scope: `--account`, `--category`, `--payee`, or `--month`. They also accept `--since`, `--until`, `--type`, and `--last-knowledge`.

Complex transaction payloads use `--input <path|->`. Input matches the API request model, including either one `transaction` or a nonempty `transactions` array:

```json
{
  "transaction": {
    "account_id": "00000000-0000-0000-0000-000000000001",
    "date": "2026-01-02",
    "amount": -1250,
    "memo": "Lunch"
  }
}
```

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | Success |
| 1 | Local, input, transport, timeout, or decode failure |
| 2 | Invalid invocation |
| 3 | Missing or invalid credential setup |
| 4 | Structured API error response |

Runtime and invocation failures are JSON objects on stderr in the default output mode. Secrets are never included in error or debug output.

## SDK

`ynab-sdk` exposes typed request and response models plus services for all 44 operations in the pinned YNAB v1 contract. Enum-like response strings retain unknown server values for forward compatibility. Requests share one authenticated executor with a rolling request limiter and GET-only retries for transport failures, timeouts, HTTP 429, and HTTP 503.

```rust
use secrecy::SecretString;
use ynab_sdk::Client;

# async fn example() -> Result<(), ynab_sdk::ApiError> {
let client = Client::new(SecretString::from("runtime-token".to_owned()));
let plans = client.plans().list(false).await?;
# Ok(())
# }
```

## Contract and development

`openapi/ynab-v1.86.0.yaml` is the reviewed API snapshot. `./scripts/check-openapi.sh` fails when the upstream contract changes; model, service, fixture, and test updates must be reviewed together.

The workspace tracks the latest stable Rust release; the current minimum is Rust 1.98. CI runs formatting, strict Clippy, all-feature tests, warning-free docs, dependency policy, secret scanning, workflow linting, workflow security analysis, and the OpenAPI drift check.

## Attribution

This project is not affiliated with, endorsed by, or sponsored by YNAB. YNAB is a registered trademark of YNAB, Inc. The package name requires YNAB's written permission before public release.

## License

Licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
