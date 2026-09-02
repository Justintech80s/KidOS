use guardian_service::{
    decode_request, load_service_state, GuardianMode, GuardianRequest, NonceTracker,
    PolicySnapshot, RequestEnvelope,
};

#[test]
fn missing_valid_policy_enters_restricted_safe_mode() {
    let state = load_service_state(None, None);
    assert_eq!(state.mode, GuardianMode::RestrictedSafeMode);
    assert_eq!(state.policy.source, "strict_baseline");
}

#[test]
fn current_valid_policy_is_preferred() {
    let current = PolicySnapshot::valid("current-policy");
    let last_known = PolicySnapshot::valid("last-policy");
    let state = load_service_state(Some(current), Some(last_known));

    assert_eq!(state.mode, GuardianMode::Healthy);
    assert_eq!(state.policy.id, "current-policy");
    assert_eq!(state.policy.source, "current");
}

#[test]
fn invalid_current_policy_falls_back_to_last_known_valid() {
    let current = PolicySnapshot::invalid("current-policy");
    let last_known = PolicySnapshot::valid("last-policy");
    let state = load_service_state(Some(current), Some(last_known));

    assert_eq!(state.mode, GuardianMode::Healthy);
    assert_eq!(state.policy.id, "last-policy");
    assert_eq!(state.policy.source, "last_known_valid");
}

#[test]
fn malformed_ipc_is_rejected() {
    assert!(decode_request(br#"{\"type\":\"shell\",\"command\":123}"#).is_err());
}

#[test]
fn unknown_protocol_version_is_rejected() {
    let request = br#"{
        \"version\": 99,
        \"session_id\": \"session-1\",
        \"nonce\": \"nonce-1\",
        \"request\": {\"type\": \"guardian_status\"}
    }"#;

    assert!(decode_request(request).is_err());
}

#[test]
fn valid_typed_request_is_decoded() {
    let request = br#"{
        \"version\": 1,
        \"session_id\": \"session-1\",
        \"nonce\": \"nonce-1\",
        \"request\": {\"type\": \"guardian_status\"}
    }"#;

    let decoded = decode_request(request).unwrap();
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.session_id, "session-1");
    assert_eq!(decoded.nonce, "nonce-1");
    assert_eq!(decoded.request, GuardianRequest::GuardianStatus);
}

#[test]
fn duplicate_nonce_is_rejected_within_active_session() {
    let envelope = RequestEnvelope {
        version: 1,
        session_id: "session-1".into(),
        nonce: "nonce-1".into(),
        request: GuardianRequest::GuardianStatus,
    };

    let mut tracker = NonceTracker::default();
    assert!(tracker.accept(&envelope).is_ok());
    assert!(tracker.accept(&envelope).is_err());
}

#[test]
fn same_nonce_can_be_used_in_a_different_session() {
    let mut tracker = NonceTracker::default();
    let first = RequestEnvelope {
        version: 1,
        session_id: "session-1".into(),
        nonce: "nonce-1".into(),
        request: GuardianRequest::GuardianStatus,
    };
    let second = RequestEnvelope {
        version: 1,
        session_id: "session-2".into(),
        nonce: "nonce-1".into(),
        request: GuardianRequest::GuardianStatus,
    };

    assert!(tracker.accept(&first).is_ok());
    assert!(tracker.accept(&second).is_ok());
}
