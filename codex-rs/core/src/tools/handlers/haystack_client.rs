use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

const HAYSTACK_URL_ENV: &str = "CODEX_HAYSTACK_URL";
const SEARCH_TIMEOUT: Duration = Duration::from_secs(10);
const NFS_SYNC_INTERVAL: Duration = Duration::from_secs(300);

// Cache of workspaces that have been successfully ensured.
static ENSURED_WORKSPACES: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Last NFS sync time per workspace.
static LAST_NFS_SYNC: Lazy<Mutex<std::collections::HashMap<PathBuf, Instant>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

/// Returns the Haystack base URL from the `CODEX_HAYSTACK_URL` env var, or
/// `None` if the variable is not set (disabling haystack integration).
pub(crate) fn haystack_url() -> Option<String> {
    std::env::var(HAYSTACK_URL_ENV).ok()
}

/// Returns `true` if `CODEX_HAYSTACK_URL` is set.
pub(crate) fn is_enabled() -> bool {
    haystack_url().is_some()
}

#[derive(Serialize)]
struct WorkspaceGetRequest<'a> {
    workspace: &'a str,
}

#[derive(Serialize)]
struct WorkspaceCreateRequest<'a> {
    workspace: &'a str,
    use_global_filters: bool,
    filters: WorkspaceCreateFilters<'a>,
}

#[derive(Serialize)]
struct WorkspaceCreateFilters<'a> {
    exclude: WorkspaceExcludeFilters<'a>,
    include: &'a [&'a str],
}

#[derive(Serialize)]
struct WorkspaceExcludeFilters<'a> {
    use_git_ignore: bool,
    customized: &'a [&'a str],
}

#[derive(Serialize)]
struct WorkspaceSyncRequest<'a> {
    workspace: &'a str,
}

#[derive(Deserialize)]
struct HaystackResponse {
    code: i32,
    #[serde(default)]
    message: Option<String>,
}

const DEFAULT_EXCLUDES: &[&str] = &[
    "**/out/**",
    "**/build/**",
    "**/node_modules/**",
    "**/.git/**",
    "**/target/**",
    "**/dist/**",
    "**/vendor/**",
    "**/third_party/**",
    "**/*.pyc",
    "**/*.o",
    "**/*.so",
    "**/*.a",
    "**/*.dylib",
    "**/*.png",
    "**/*.jpg",
    "**/*.gif",
    "**/*.ico",
    "**/*.woff",
    "**/*.woff2",
    "**/*.ttf",
    "**/*.zip",
    "**/*.tar",
    "**/*.gz",
    "**/*.pdf",
];

/// Idempotently ensures a Haystack workspace exists for `workspace_path`.
///
/// On first call for a given path the function checks whether the workspace
/// already exists via `/workspace/get`, creating it via `/workspace/create`
/// if not.  Results are cached for the process lifetime so subsequent calls
/// are zero-cost.
pub(crate) async fn ensure_workspace(workspace_path: &Path) {
    // Fast path: already ensured in this process.
    {
        let cache = ENSURED_WORKSPACES.lock().unwrap();
        if cache.contains(workspace_path) {
            return;
        }
    }

    let Some(base_url) = haystack_url() else {
        return;
    };

    let workspace_str = workspace_path.to_string_lossy();
    let client = reqwest::Client::new();

    // Check if workspace already exists.
    let get_url = format!("{base_url}/api/v1/workspace/get");
    let get_body = WorkspaceGetRequest {
        workspace: &workspace_str,
    };
    let workspace_exists = match client.post(&get_url).json(&get_body).send().await {
        Ok(resp) => {
            if let Ok(parsed) = resp.json::<HaystackResponse>().await {
                parsed.code == 0
            } else {
                false
            }
        }
        Err(e) => {
            tracing::debug!("haystack workspace/get failed: {e}");
            false
        }
    };

    if !workspace_exists {
        let create_url = format!("{base_url}/api/v1/workspace/create");
        let create_body = WorkspaceCreateRequest {
            workspace: &workspace_str,
            use_global_filters: true,
            filters: WorkspaceCreateFilters {
                exclude: WorkspaceExcludeFilters {
                    use_git_ignore: true,
                    customized: DEFAULT_EXCLUDES,
                },
                include: &["**/*"],
            },
        };
        match client.post(&create_url).json(&create_body).send().await {
            Ok(resp) => {
                if let Ok(parsed) = resp.json::<HaystackResponse>().await {
                    if parsed.code != 0 {
                        let msg = parsed.message.unwrap_or_default();
                        // "Workspace already exists" is a benign race condition.
                        if !msg.contains("already exists") {
                            tracing::warn!("haystack workspace/create returned error: {msg}");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::debug!("haystack workspace/create failed: {e}");
                return;
            }
        }
    }

    // Mark as ensured.
    let mut cache = ENSURED_WORKSPACES.lock().unwrap();
    cache.insert(workspace_path.to_path_buf());
}

/// Returns `true` if `path` is on a network filesystem (NFS, CIFS, AFS,
/// Lustre) by reading `/proc/mounts`.
pub(crate) fn is_network_fs(path: &Path) -> bool {
    let Ok(mounts) = std::fs::read_to_string("/proc/mounts") else {
        return false;
    };
    let path_str = path.to_string_lossy();
    let mut best_mount_len = 0usize;
    let mut best_is_network = false;
    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let _ = parts.next(); // device
        let mount_point = parts.next().unwrap_or("");
        let fs_type = parts.next().unwrap_or("");
        if path_str.starts_with(mount_point) && mount_point.len() > best_mount_len {
            best_mount_len = mount_point.len();
            best_is_network =
                matches!(fs_type, "nfs" | "nfs4" | "cifs" | "afs" | "lustre" | "gpfs");
        }
    }
    best_is_network
}

/// If `path` is on a network filesystem and the last sync for this workspace
/// was more than 5 minutes ago, fires a background sync (non-blocking).
pub(crate) async fn maybe_sync_nfs_workspace(path: &Path) {
    if !is_network_fs(path) {
        return;
    }
    let needs_sync = {
        let map = LAST_NFS_SYNC.lock().unwrap();
        map.get(path)
            .map(|t| t.elapsed() > NFS_SYNC_INTERVAL)
            .unwrap_or(true)
    };
    if !needs_sync {
        return;
    }
    {
        let mut map = LAST_NFS_SYNC.lock().unwrap();
        map.insert(path.to_path_buf(), Instant::now());
    }

    let Some(base_url) = haystack_url() else {
        return;
    };
    let workspace_str = path.to_string_lossy().to_string();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let url = format!("{base_url}/api/v1/workspace/sync");
        let body = WorkspaceSyncRequest {
            workspace: &workspace_str,
        };
        if let Err(e) = client.post(&url).json(&body).send().await {
            tracing::debug!("haystack workspace/sync failed: {e}");
        }
    });
}

// Search API types.

#[derive(Serialize)]
struct SearchRequest<'a> {
    workspace: &'a str,
    query: &'a str,
    case_sensitive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<SearchFilters<'a>>,
    limit: SearchLimit,
}

#[derive(Serialize)]
struct SearchFilters<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<&'a str>,
}

#[derive(Serialize)]
struct SearchLimit {
    max_results: usize,
    max_results_per_file: usize,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    code: i32,
    data: Option<SearchData>,
}

#[derive(Deserialize, Debug)]
struct SearchData {
    results: Vec<SearchResult>,
    // Part of the Haystack API contract; indicates whether results were
    // truncated to `max_results`. Not used in current logic.
    truncate: bool,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    file: String,
}

/// Runs a search against the Haystack server. Returns `Vec<String>` of file
/// paths in the same format as `run_rg_search` (relative paths from `cwd`
/// when possible, otherwise absolute).
///
/// Returns `Err` on any error (network, timeout, bad response) so the caller
/// can fall through to `rg`.
pub(crate) async fn run_haystack_search(
    pattern: &str,
    include: Option<&str>,
    search_path: &Path,
    limit: usize,
    cwd: &Path,
) -> anyhow::Result<Vec<String>> {
    let base_url = haystack_url().ok_or_else(|| anyhow::anyhow!("haystack not configured"))?;

    let workspace_str = search_path.to_string_lossy();
    let client = reqwest::Client::new();
    let url = format!("{base_url}/api/v1/search/content");

    let filters = include.map(|inc| SearchFilters { include: Some(inc) });

    let body = SearchRequest {
        workspace: &workspace_str,
        query: pattern,
        case_sensitive: false,
        filters,
        limit: SearchLimit {
            max_results: limit,
            max_results_per_file: 50,
        },
    };

    let resp = tokio::time::timeout(SEARCH_TIMEOUT, client.post(&url).json(&body).send())
        .await
        .map_err(|_| anyhow::anyhow!("haystack search timed out"))?
        .map_err(|e| anyhow::anyhow!("haystack request failed: {e}"))?;

    let parsed: SearchResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("failed to parse haystack response: {e}"))?;

    if parsed.code != 0 {
        return Err(anyhow::anyhow!(
            "haystack returned non-zero code: {}",
            parsed.code
        ));
    }

    let data = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("haystack response missing data field"))?;

    // If the workspace is still indexing and returned no results, signal the
    // caller to fall through to rg.
    if data.results.is_empty() {
        return Err(anyhow::anyhow!(
            "haystack returned zero results (may still be indexing)"
        ));
    }

    let paths: Vec<String> = data
        .results
        .into_iter()
        .map(|r| {
            let file_path = PathBuf::from(&r.file);
            // Return relative path from cwd when possible.
            file_path
                .strip_prefix(cwd)
                .map(|rel| rel.to_string_lossy().to_string())
                .unwrap_or(r.file)
        })
        .collect();

    Ok(paths)
}
