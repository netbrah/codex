//! Unit tests for the request_permissions handler (Ratchet D projection guard).

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn request_permissions_environment_args_stays_tolerant_projection() {
    let args: RequestPermissionsEnvironmentArgs =
        serde_json::from_str(r#"{"environment_id":"env-1","bogus":1}"#)
            .expect("the environment pre-parse must stay a tolerant projection");
    assert_eq!(args.environment_id.as_deref(), Some("env-1"));

    let args: RequestPermissionsEnvironmentArgs =
        serde_json::from_str(r#"{"environmentId":"env-2"}"#)
            .expect("the camelCase alias must keep parsing");
    assert_eq!(args.environment_id.as_deref(), Some("env-2"));
}
