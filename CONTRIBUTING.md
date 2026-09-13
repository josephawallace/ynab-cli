# Contributing

Use the latest stable Rust release, currently Rust 1.98. When adopting a new stable release, update `rust-version` with it; compatibility with older toolchains is not a project goal. Do not use a real account or access token in tests, fixtures, issues, or commits. Contract changes update the versioned OpenAPI snapshot, typed models, services, fixtures, tests, and changelog together.

Before submitting, run the repository verification commands documented in CI. Keep public models forward-compatible: newly introduced server enum values must decode rather than fail.
