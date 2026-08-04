
## Review fixes — 2026-08-04

### Findings addressed

1. Removed unused `dialog:default` capability. Dialog plugin remains registered for future explicit permissions, but current default capability grants only core defaults and required filesystem reads.
2. Replaced `csp: null` with restrictive policy: self-hosted static assets/scripts, Tauri IPC connections, data images/fonts, inline styles required by the smoke page, and blocked objects, base URLs, and framing.
3. Defined typed command convention in `src/lib/commands.ts`: command payloads are optional `Record<string, unknown>` objects whose keys match Rust command argument names; no-argument commands omit payload. `createCommandInvoker` provides the injectable implementation and `invokeCommand` is the production instance. Current `ping` uses omitted payload.
4. Replaced source-regex test with behavioral tests that inject an invoke implementation and assert exact command, payload, result, and no-argument behavior.

### Validation

- `npm run test`: passed, 2 tests.
- `npm run typecheck`: passed.
- `npm run build`: passed; static routes `/` and `/_not-found` generated in `out/`.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed; 1 Rust unit test and 0 doc tests failed.
- Generated capability manifest confirms `dialog:default` is absent.

### Concerns

- Native window runtime/build packaging remains untested on Windows, macOS, Android, and iOS from this Linux environment; cross-platform packaging validation intentionally deferred.
