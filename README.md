# req

`req` is a terminal-native API client (Rust + ratatui) for testing HTTP APIs from your terminal.

## Install on macOS (no Cargo required)

`req` ships prebuilt macOS binaries on GitHub Releases.

```bash
curl -fsSL https://raw.githubusercontent.com/muneerulhudha/req/main/install.sh | bash
```

Then verify:

```bash
req --help
```

### Installer options

- `REQ_INSTALL_OWNER` (default: `muneerulhudha`)
- `REQ_INSTALL_REPO` (default: `req`)
- `REQ_INSTALL_DIR` (default: `/usr/local/bin`)

Example installing from a fork without sudo:

```bash
curl -fsSL https://raw.githubusercontent.com/muneerulhudha/req/main/install.sh | \
  REQ_INSTALL_OWNER=<YOUR_GITHUB_OWNER> REQ_INSTALL_REPO=req REQ_INSTALL_DIR="$HOME/.local/bin" bash
```

## Build from source (development)

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
