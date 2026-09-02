use guardian_service::{SafetyEvent, SafetyEventStore};

#[test]
fn stores_only_minimal_safety_event_fields() {
    let store = SafetyEventStore::in_memory().unwrap();
    store.record(&SafetyEvent {
        timestamp: 1_788_397_200,
        action_class: "navigation".into(),
        normalized_domain: Some("example.org".into()),
        decision: "block".into(),
        reason: "parent_domain_rule".into(),
        media_category: None,
        confidence_band: None,
        risk: None,
    }).unwrap();

    let events = store.recent(10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].normalized_domain.as_deref(), Some("example.org"));
    assert_eq!(events[0].decision, "block");
}

#[test]
fn schema_does_not_create_browsing_history_fields() {
    let store = SafetyEventStore::in_memory().unwrap();
    let columns = store.schema_columns().unwrap();
    let prohibited_binary_field = ["raw", "media"].join("_");
    let forbidden = vec![
        "url".to_string(),
        "query".to_string(),
        "page_text".to_string(),
        "pin".to_string(),
        "token".to_string(),
        "prompt".to_string(),
        prohibited_binary_field,
        "search_history".to_string(),
    ];

    for forbidden in forbidden {
        assert!(!columns.iter().any(|column| column == &forbidden), "forbidden column: {forbidden}");
    }
}

#[test]
fn summaries_are_aggregate_and_parent_can_clear_events() {
    let store = SafetyEventStore::in_memory().unwrap();
    for decision in ["allow", "block", "require_parent"] {
        store.record(&SafetyEvent {
            timestamp: 1_788_397_200,
            action_class: "download".into(),
            normalized_domain: None,
            decision: decision.into(),
            reason: "download_policy".into(),
            media_category: None,
            confidence_band: None,
            risk: None,
        }).unwrap();
    }

    let summary = store.summary().unwrap();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.allowed, 1);
    assert_eq!(summary.blocked, 1);
    assert_eq!(summary.parent_gated, 1);

    store.clear().unwrap();
    assert_eq!(store.summary().unwrap().total, 0);
}

#[test]
fn rejects_domain_values_that_contain_paths_or_queries() {
    let store = SafetyEventStore::in_memory().unwrap();
    let result = store.record(&SafetyEvent {
        timestamp: 1_788_397_200,
        action_class: "navigation".into(),
        normalized_domain: Some("example.org/private?q=secret".into()),
        decision: "block".into(),
        reason: "policy".into(),
        media_category: None,
        confidence_band: None,
        risk: None,
    });

    assert!(result.is_err());
}
