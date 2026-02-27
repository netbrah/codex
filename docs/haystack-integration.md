# Haystack Integration

[Haystack](https://github.com/CodeTrek/haystack) is an optional, pre-indexed code search server that Codex can use to accelerate the `grep_files` tool. On large repositories (100k+ files) where ripgrep may be slow, Haystack maintains a persistent index so searches return in milliseconds.

## How It Works

When the `CODEX_HAYSTACK_URL` environment variable is set, Codex will:

1. **Register the workspace** with Haystack on the first search (creating the index if needed).
2. **Search via Haystack** for each `grep_files` call.
3. **Fall back to ripgrep** automatically if Haystack is unavailable or returns an error.

When `CODEX_HAYSTACK_URL` is **not** set, Codex behaves exactly as before — all searches go through ripgrep. There is zero behavior change for users who don't configure Haystack.

## Setup

### 1. Install Haystack

Run the provided setup script:

```bash
./scripts/haystack-setup.sh
```

This will:
- Download the Haystack binary for your platform (Linux amd64/arm64 only).
- Write a default configuration to `~/.haystack/config.yaml`.
- Start the server on port 13135.

You can customize the port and data directory:

```bash
./scripts/haystack-setup.sh --port 13136 --data-dir /opt/haystack
```

### 2. Set the Environment Variable

Add this to your shell profile (e.g., `~/.bashrc`, `~/.zshrc`):

```bash
export CODEX_HAYSTACK_URL="http://127.0.0.1:13135"
```

### 3. Verify

Check that Haystack is running:

```bash
curl http://127.0.0.1:13135/health
```

Then run Codex normally — the `grep_files` tool will automatically use Haystack.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `CODEX_HAYSTACK_URL` | *(unset)* | Base URL of the Haystack server. When set, enables the integration. |

The Haystack server itself is configured via `~/.haystack/config.yaml`. See the [Haystack documentation](https://github.com/CodeTrek/haystack) for details.

## Architecture

```
grep_files tool invoked
        │
        ▼
  CODEX_HAYSTACK_URL set?
   │                │
  NO               YES
   │                │
   ▼                ▼
run ripgrep    ensure_workspace()
(unchanged)         │
   │                ▼
   │         haystack search
   │           │         │
   │        success    failure
   │           │         │
   │           ▼         ▼
   │      return      run ripgrep
   │      results     (fallback)
   ▼           ▼         ▼
     Results returned to model
```

## Troubleshooting

- **Haystack not found**: Make sure the server is running and `CODEX_HAYSTACK_URL` points to the correct address.
- **Slow first search**: The first search after starting Codex triggers workspace registration and indexing. Subsequent searches are fast.
- **NFS/network mounts**: Haystack periodically syncs the index on network-mounted file systems (every 5 minutes by default).
- **Falling back to ripgrep**: If you see `haystack search failed, falling back to rg` in logs, check that the Haystack server is healthy.

## Supported Platforms

Haystack is a statically-linked Go binary. It currently supports:

- Linux amd64
- Linux arm64
