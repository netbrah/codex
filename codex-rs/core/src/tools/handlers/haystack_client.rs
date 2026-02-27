//! HTTP client for the Haystack code search server.
//!
//! Provides workspace management, NFS detection, and search with graceful error
//! handling. All functions are no-ops when `CODEX_HAYSTACK_URL` is unset.

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

// ── Timeouts ──

const HAYSTACK_SEARCH_TIMEOUT: Duration = Duration::from_secs(10);
const HAYSTACK_WORKSPACE_TIMEOUT: Duration = Duration::from_secs(5);
const NFS_SYNC_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
const MAX_RESULTS_PER_FILE: usize = 50;

// ── Statics ──

static HAYSTACK_ENABLED: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("CODEX_HAYSTACK_URL")
        .ok()
        .is_some_and(|v| !v.is_empty())
});

static HAYSTACK_URL: LazyLock<Option<String>> = LazyLock::new(|| {
    std::env::var("CODEX_HAYSTACK_URL")
        .ok()
        .filter(|v| !v.is_empty())
});

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(HAYSTACK_SEARCH_TIMEOUT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

static INITIALIZED_WORKSPACES: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static LAST_SYNC: LazyLock<Mutex<HashMap<PathBuf, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ── Request types ──

#[derive(Serialize)]
struct SearchContentRequest {
    workspace: String,
    query: String,
    case_sensitive: bool,
    filters: SearchFilters,
    limit: SearchLimit,
}

#[derive(Serialize)]
struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude: Option<String>,
}

#[derive(Serialize)]
struct SearchLimit {
    max_results: usize,
    max_results_per_file: usize,
}

#[derive(Serialize)]
struct WorkspaceRequest {
    workspace: String,
}

#[derive(Serialize)]
struct CreateWorkspaceRequest {
    workspace: String,
    use_global_filters: bool,
    filters: CreateWorkspaceFilters,
}

#[derive(Serialize)]
struct CreateWorkspaceFilters {
    exclude: ExcludeFilters,
    include: Vec<String>,
}

#[derive(Serialize)]
struct ExcludeFilters {
    use_git_ignore: bool,
    customized: Vec<String>,
}

// ── Response types ──

#[derive(Deserialize)]
struct HaystackResponse<T> {
    code: i64,
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Default, Deserialize)]
struct SearchData {
    results: Vec<SearchResult>,
    #[serde(default)]
    truncate: bool,
}

#[derive(Deserialize)]
struct SearchResult {
    file: String,
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
struct WorkspaceData {
    #[serde(default)]
    indexing: bool,
    #[serde(default)]
    total_files: u64,
    #[serde(default)]
    indexed_files: u64,
}

// ── Public API ──

/// Returns `true` if `CODEX_HAYSTACK_URL` is set and non-empty.
pub fn is_enabled() -> bool {
    *HAYSTACK_ENABLED
}

fn haystack_url() -> Option<&'static str> {
    HAYSTACK_URL.as_deref()
}

/// Ensure the workspace is registered with Haystack. Idempotent — repeated
/// calls for the same path are fast no-ops after the first success.
pub async fn ensure_workspace(workspace_path: &Path) -> Result<(), String> {
    let url = haystack_url().ok_or_else(|| "Haystack not configured".to_string())?;

    // Fast path: already initialised.
    {
        let cache = INITIALIZED_WORKSPACES
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;
        if cache.contains(workspace_path) {
            return Ok(());
        }
    }

    let ws = workspace_path.to_string_lossy().to_string();

    // Check whether the workspace already exists.
    let get_resp: HaystackResponse<WorkspaceData> = HTTP_CLIENT
        .post(format!("{url}/api/v1/workspace/get"))
        .timeout(HAYSTACK_WORKSPACE_TIMEOUT)
        .json(&WorkspaceRequest {
            workspace: ws.clone(),
        })
        .send()
        .await
        .map_err(|e| format!("haystack workspace/get request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("haystack workspace/get parse failed: {e}"))?;

    if get_resp.code == 0 {
        // Workspace exists — cache and return.
        let mut cache = INITIALIZED_WORKSPACES
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;
        cache.insert(workspace_path.to_path_buf());
        maybe_trigger_nfs_sync(url, workspace_path);
        return Ok(());
    }

    // Workspace does not exist (code == 1) — create it.
    let create_resp: HaystackResponse<WorkspaceData> = HTTP_CLIENT
        .post(format!("{url}/api/v1/workspace/create"))
        .timeout(HAYSTACK_WORKSPACE_TIMEOUT)
        .json(&CreateWorkspaceRequest {
            workspace: ws,
            use_global_filters: true,
            filters: CreateWorkspaceFilters {
                exclude: ExcludeFilters {
                    use_git_ignore: true,
                    customized: vec![
                        "node_modules".to_string(),
                        ".git".to_string(),
                        "target".to_string(),
                        "dist".to_string(),
                        "build".to_string(),
                    ],
                },
                include: vec!["**/*".to_string()],
            },
        })
        .send()
        .await
        .map_err(|e| format!("haystack workspace/create request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("haystack workspace/create parse failed: {e}"))?;

    if create_resp.code != 0 {
        return Err(format!(
            "haystack workspace/create failed: {}",
            create_resp.message.unwrap_or_default()
        ));
    }

    let mut cache = INITIALIZED_WORKSPACES
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;
    cache.insert(workspace_path.to_path_buf());
    Ok(())
}

/// Search for `pattern` via Haystack, returning a list of matching file paths.
///
/// Returns `Err` if the HTTP call fails or the response is malformed; the
/// caller should fall back to ripgrep in that case.
pub async fn search(
    workspace_path: &Path,
    pattern: &str,
    include: Option<&str>,
    limit: usize,
) -> Result<Vec<String>, String> {
    let url = haystack_url().ok_or_else(|| "Haystack not configured".to_string())?;

    let ws = workspace_path.to_string_lossy().to_string();

    let resp: HaystackResponse<SearchData> = HTTP_CLIENT
        .post(format!("{url}/api/v1/search/content"))
        .timeout(HAYSTACK_SEARCH_TIMEOUT)
        .json(&SearchContentRequest {
            workspace: ws,
            query: pattern.to_string(),
            case_sensitive: false,
            filters: SearchFilters {
                include: include.map(str::to_string),
                exclude: None,
            },
            limit: SearchLimit {
                max_results: limit,
                max_results_per_file: MAX_RESULTS_PER_FILE,
            },
        })
        .send()
        .await
        .map_err(|e| format!("haystack search request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("haystack search parse failed: {e}"))?;

    if resp.code != 0 {
        return Err(format!(
            "haystack search failed: {}",
            resp.message.unwrap_or_default()
        ));
    }

    let data = resp
        .data
        .ok_or_else(|| "haystack search returned no data".to_string())?;

    // De-duplicate file paths (Haystack may return multiple matches per file).
    let mut seen = HashSet::new();
    let files: Vec<String> = data
        .results
        .into_iter()
        .filter_map(|r| {
            if seen.insert(r.file.clone()) {
                Some(r.file)
            } else {
                None
            }
        })
        .take(limit)
        .collect();

    if data.truncate {
        tracing::debug!("haystack search results were truncated");
    }

    Ok(files)
}

// ── NFS sync helper ──

/// Fire-and-forget NFS sync — called at most once per `NFS_SYNC_INTERVAL` per
/// workspace to keep Haystack's index fresh on network-mounted file systems.
fn maybe_trigger_nfs_sync(url: &'static str, workspace_path: &Path) {
    let path_buf = workspace_path.to_path_buf();
    let should_sync = {
        let mut map = match LAST_SYNC.lock() {
            Ok(m) => m,
            Err(_) => return,
        };
        let now = Instant::now();
        match map.get(&path_buf) {
            Some(last) if now.duration_since(*last) < NFS_SYNC_INTERVAL => false,
            _ => {
                map.insert(path_buf.clone(), now);
                true
            }
        }
    };

    if should_sync {
        let ws = path_buf.to_string_lossy().to_string();
        tokio::spawn(async move {
            let _ = HTTP_CLIENT
                .post(format!("{url}/api/v1/workspace/sync"))
                .timeout(HAYSTACK_WORKSPACE_TIMEOUT)
                .json(&WorkspaceRequest { workspace: ws })
                .send()
                .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_enabled_reflects_env() {
        // In the test environment CODEX_HAYSTACK_URL is not set, so the cached
        // value should be false.  We cannot re-test with a set variable because
        // LazyLock is process-global, but we can at least assert the default.
        assert!(!*HAYSTACK_ENABLED);
    }

    #[test]
    fn haystack_url_returns_none_when_unset() {
        assert!(haystack_url().is_none());
    }

    #[test]
    fn search_filters_skips_none() {
        let filters = SearchFilters {
            include: None,
            exclude: None,
        };
        let json = serde_json::to_string(&filters).unwrap_or_default();
        assert_eq!(json, "{}");
    }

    #[test]
    fn search_filters_includes_present_values() {
        let filters = SearchFilters {
            include: Some("*.rs".to_string()),
            exclude: None,
        };
        let json = serde_json::to_string(&filters).unwrap_or_default();
        assert!(json.contains("*.rs"));
        assert!(!json.contains("exclude"));
    }
}
