# Localhost Custom-Port Base URL Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow OpenAI-compatible local AI servers at `localhost` or `127.0.0.1` with custom ports and paths such as `http://localhost:20128/v1`.

**Architecture:** Extend native AI URL validation with an explicit loopback exception while retaining all existing URL and SSRF checks for non-loopback hosts. Reuse existing endpoint joining and pinned socket behavior so `/v1` becomes `/v1/chat/completions` without changing frontend request contracts.

**Tech Stack:** Rust, Tauri, reqwest, Url, Tokio, existing Rust integration tests.

## Global Constraints

- Permit `localhost` and loopback `127.0.0.1` hosts.
- Preserve custom port and path, including `/v1`.
- OpenAI-compatible requests append `/chat/completions` without duplicating an existing suffix.
- Keep URL safety checks for credentials, query strings, fragments, unsupported schemes, and malformed URLs.
- Continue rejecting non-loopback private/LAN addresses such as `10.x.x.x`, `172.16.x.x`, `192.168.x.x`, and link-local addresses.
- No new dependencies.
- Do not weaken public-host SSRF protections.

---

### Task 1: Add failing localhost URL coverage

**Files:**
- Modify: `src-tauri/tests/ai.rs`
- Test: `src-tauri/tests/ai.rs`

**Interfaces:**
- Consumes existing `ai::endpoint`, `ai::endpoint_for_client`, and `ai::endpoint_with_resolver` APIs.
- Produces regression tests defining accepted loopback URLs and rejected non-loopback private results.

- [ ] **Step 1: Add tests for localhost custom port and path**

Add tests asserting:

```rust
#[test]
fn localhost_custom_port_base_url_is_supported() {
    let endpoint = ai::endpoint_for_client(
        "http://localhost:20128/v1",
        "chat/completions",
    )
    .unwrap();
    assert_eq!(endpoint.as_str(), "http://localhost:20128/v1/chat/completions");

    let loopback = ai::endpoint_with_resolver(
        "http://127.0.0.1:20128/v1",
        "chat/completions",
        || Ok::<_, std::io::Error>(vec!["127.0.0.1".parse().unwrap()]),
    )
    .unwrap();
    assert_eq!(loopback.as_str(), "http://127.0.0.1:20128/v1/chat/completions");
}
```

Add a test using `endpoint_with_resolver` that confirms a non-loopback private result remains rejected:

```rust
#[test]
fn localhost_resolution_rejects_non_loopback_result() {
    let result = ai::endpoint_with_resolver(
        "http://localhost:20128/v1",
        "chat/completions",
        || Ok::<_, std::io::Error>(vec!["192.168.1.20".parse().unwrap()]),
    );
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run focused tests and verify the new localhost test fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test ai localhost_custom_port_base_url_is_supported localhost_resolution_rejects_non_loopback_result
```

Expected: the localhost test fails with the existing unsafe/private URL validation error, while the private-result rejection test remains passing.

---

### Task 2: Implement loopback exception without weakening SSRF checks

**Files:**
- Modify: `src-tauri/src/ai/mod.rs:494-525,548-575`
- Test: `src-tauri/tests/ai.rs`

**Interfaces:**
- Keeps `endpoint_with_resolver(base_url: &str, suffix: &str, resolver: F) -> Result<Url, AppError>` unchanged.
- Keeps `resolve_and_pin(base_url: &str, resolver: F) -> Result<(Url, SocketAddr), AppError>` unchanged.
- Adds internal loopback classification helpers only.

- [ ] **Step 1: Add explicit loopback host classification**

Implement an internal helper that returns true only for:

```rust
url.host_str() == Some("localhost")
    || url.host_str() == Some("127.0.0.1")
    || url.host_str() == Some("[::1]")
```

Use normalized host comparison rather than raw URL string comparison so host casing does not bypass validation.

- [ ] **Step 2: Update endpoint validation**

In `endpoint_with_resolver`, keep scheme, host, credentials, query, and fragment validation unchanged. Permit loopback hosts through the hostname private check, then require resolver results to be loopback-only. Reject a resolver result if any address is non-loopback, private non-loopback, link-local, or otherwise unsafe.

Do not permit arbitrary private IPv4/IPv6 hosts. The existing `is_private_ip` checks must continue to reject LAN and metadata-network addresses.

- [ ] **Step 3: Update pinned resolution consistently**

Apply the same loopback-only rule in `resolve_and_pin`, ensuring the selected pinned socket is loopback when the configured host is localhost/127.0.0.1. Preserve the existing first-address selection for public hosts.

- [ ] **Step 4: Run focused tests and verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test ai localhost_custom_port_base_url_is_supported localhost_resolution_rejects_non_loopback_result
```

Expected: PASS.

---

### Task 3: Verify existing safety and application behavior

**Files:**
- Modify: none unless test adjustments are required
- Test: `src-tauri/tests/ai.rs`, `tests/components.test.mjs`

- [ ] **Step 1: Run full frontend tests**

```bash
npm test
```

Expected: all tests pass.

- [ ] **Step 2: Run lint and typecheck**

```bash
npm run lint
npm run typecheck
```

Expected: both commands pass.

- [ ] **Step 3: Run all Rust tests and formatting checks**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
git diff --check
```

Expected: all commands pass; existing public URL, private-network, credential, query, fragment, and endpoint-joining tests remain green.

- [ ] **Step 4: Inspect final diff**

```bash
git diff -- src-tauri/src/ai/mod.rs src-tauri/tests/ai.rs
```

Confirm only localhost validation and its regression coverage changed; no API keys, credentials, or unrelated generated artifacts are included.
