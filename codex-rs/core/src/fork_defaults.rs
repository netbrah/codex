//! Fork-local defaults for the codex-combined on-prem build (FORK-MANIFEST tracked).
//!
//! This file is a fork seam: it exists in this repository only and MUST NOT be
//! merged upstream. Operator ruling 2026-09-17: this binary is the on-prem
//! workhorse, and proactive multi-agent delegation is its default policy
//! (no new config flag; see docs/onprem-mode-catalog-spec.md).

/// Proactive delegation is the default mode in this build when no
/// `multi_agent_mode_hint_text` override is configured (any reasoning effort).
/// Upstream builds keep the effort-derived selection (Ultra => Proactive,
/// otherwise ExplicitRequestOnly).
pub(crate) fn proactive_delegation_default() -> bool {
    true
}
