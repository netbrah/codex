# Haystack Integration

Codex can use [Haystack](https://github.com/netbrah/haystack-vscode) — a
fast, pre-indexed code search server — as the backend for the `grep_files`
tool.  On large repositories (100k+ files) Haystack's PebbleDB index makes
searches orders-of-magnitude faster than walking the file tree with ripgrep
on every query.

When Haystack is not configured, Codex falls back to the standard `rg`
backend with **zero behaviour change**.

## How it works

```
grep_files tool call
       │
       ▼
 CODEX_HAYSTACK_URL set?
       │
  yes  │  no
       │───────────────────────────────────────────────┐
       ▼                                               ▼
ensure_workspace()          run_rg_search() (unchanged behaviour)
       │
       ▼
run_haystack_search()
  POST /api/v1/search/content
       │
  success & results?
       │
  yes  │  no / error
       │───────────┐
       ▼           ▼
 return results  run_rg_search() (transparent fallback)
```

The Haystack server runs as a local HTTP server on `127.0.0.1` (default port
`13136`).  On first use for a given workspace path, Codex calls
`/api/v1/workspace/create` to register the directory so Haystack can build
and maintain its index incrementally.

## Setup

1. Obtain the Haystack binary for your platform (`linux-amd64` or
   `linux-arm64`) and place it at `scripts/bin/haystack`.

2. Run the setup script once:

   ```sh
   ./scripts/haystack-setup.sh
   ```

   The script:
   - Installs the binary to `~/.haystack-codex/bin/`
   - Detects NFS home directories and redirects the index data to local disk
     (see [NFS considerations](#nfs-considerations))
   - Writes `~/.haystack-codex/config.yaml` with port `13136`
   - Installs and starts a `systemd --user` service (`haystack-codex.service`)
     that automatically restarts after reboots
   - Appends `export CODEX_HAYSTACK_URL=http://127.0.0.1:13136` to
     `~/.bashrc` and/or `~/.zshrc`

3. Reload your shell or run:

   ```sh
   export CODEX_HAYSTACK_URL=http://127.0.0.1:13136
   ```

After the first `grep_files` call for a workspace, Haystack will begin
indexing in the background.  Subsequent searches within the same process are
cached so workspace registration has zero overhead.

## Configuration

| Environment variable    | Description                                                 | Default        |
|-------------------------|-------------------------------------------------------------|----------------|
| `CODEX_HAYSTACK_URL`    | Base URL of the Haystack server. Leave unset to disable.   | *(unset → rg)* |

### `.haystackignore`

Haystack respects `.gitignore` files automatically (`use_git_ignore: true`).
The following additional patterns are always excluded from the index:

```
**/out/**  **/build/**  **/node_modules/**  **/.git/**  **/target/**
**/dist/** **/vendor/** **/third_party/**
*.pyc  *.o  *.so  *.a  *.dylib
*.png  *.jpg  *.gif  *.ico
*.woff  *.woff2  *.ttf
*.zip  *.tar  *.gz  *.pdf
```

## NFS considerations

`inotify` (which Haystack uses to watch for file changes) does not work on
network filesystems such as NFS, CIFS, AFS, or Lustre.  Codex detects this
automatically by inspecting `/proc/mounts`:

- **Index data** is redirected to local disk during `haystack-setup.sh` (it
  tries `/local/$USER`, `/scratch/$USER`, `/var/tmp/$USER-haystack-codex`,
  `/tmp/$USER-haystack-codex` in order).
- **Periodic syncs** — on every `grep_files` call where the workspace is on a
  network filesystem, Codex fires a background `/api/v1/workspace/sync` POST
  no more than once every 5 minutes so the index stays fresh without
  hammering the server.

## Troubleshooting

| Symptom | Command |
|---------|---------|
| Check service status | `systemctl --user status haystack-codex` |
| View logs | `journalctl --user -u haystack-codex -f` |
| Manual health check | `curl http://127.0.0.1:13136/health` |
| Restart service | `systemctl --user restart haystack-codex` |
| Force re-index a workspace | `curl -s -X POST http://127.0.0.1:13136/api/v1/workspace/sync -H 'Content-Type: application/json' -d '{"workspace":"/path/to/repo"}'` |

If Haystack is unavailable or returns an error, Codex automatically falls
back to `rg`.  You will see a `DEBUG`-level log line:

```
haystack search failed, falling back to rg: <reason>
```

Enable debug logging with `RUST_LOG=codex_core=debug` to see these messages.
