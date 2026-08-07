# Localhost Custom-Port Base URL Support

## Goal
Allow NutriSurvey Pro to connect to local OpenAI-compatible AI servers using a custom Base URL such as `http://localhost:20128/v1`.

## Scope
- Permit `localhost` and loopback `127.0.0.1` hosts.
- Preserve custom port and path, including `/v1`.
- OpenAI-compatible requests append `/chat/completions` without duplicating an existing suffix.
- Keep URL safety checks for credentials, query strings, fragments, unsupported schemes, and malformed URLs.
- Continue rejecting non-loopback private/LAN addresses such as `10.x.x.x`, `172.16.x.x`, `192.168.x.x`, and link-local addresses.

## Design
The native AI URL validation layer will classify loopback hosts as an explicit local-development exception before private-address rejection. DNS resolution for localhost must still resolve only to loopback addresses; any mixed or non-loopback result is rejected. Existing pinned-socket behavior remains in place.

OpenAI-compatible providers (`openai`, `openrouter`, and `custom`) use the configured Base URL and append `chat/completions` through the existing endpoint joiner. A Base URL ending in `/v1` therefore becomes `http://localhost:20128/v1/chat/completions`.

## Error Handling
Invalid localhost URLs return the existing safe validation error. Localhost connection failures return the existing AI request error without exposing API keys. No credentials may appear in the URL.

## Testing
Add tests covering:
- `http://localhost:20128/v1` validation and endpoint construction.
- `http://127.0.0.1:20128/v1` validation and endpoint construction.
- Rejection of private LAN addresses.
- Rejection of query, fragment, credentials, and non-HTTP schemes.
- Existing public-provider and SSRF protections remain passing.
