# req

`req` is a terminal-native API client (Rust + ratatui) for testing HTTP APIs from your terminal.

## Install once, run as `req`

```bash
cargo install --path .
req
```

## Features

- Full HTTP method support: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, TRACE, CONNECT.
- Three-pane TUI:
  - **Left:** collections and history
  - **Middle:** request editor (name, method, URL, params, headers, body, env)
  - **Right:** response body/headers viewer
- Environment interpolation with `{{var}}` syntax.
- Local persistence at `~/.config/req/state.json`.
- cURL preview generation.
- Pretty-printed JSON response bodies.

## Controls

- `Tab`: cycle focus (collections → history → editor)
- `↑/↓`: navigate selected list/editor field
- `e`: edit selected editor field
- `m`: cycle HTTP method
- `s`: send request
- `w`: save request to collection
- `l`: load selected collection (when collections pane focused)
- `h`: load selected history request/response (when history pane focused)
- `n`: new request
- `c`: cURL preview
- `r`: toggle response body/headers
- `q`: quit

## Notes

Field values in params, headers, and env are line-based:

- `key: value`
- `key=value`
