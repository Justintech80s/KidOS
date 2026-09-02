use policy_core::{evaluate_media, MediaCategory, MediaContext, MediaRisk, PolicyDecision};

fn media(age: u8, category: MediaCategory, risk: MediaRisk) -> MediaContext {
    MediaContext {
        age,
        category,
        risk,
        high_confidence: true,
        parent_blocked: false,
        classifier_available: true,
        teen_uncertain_enabled: false,
    }
}

#[test]
fn explicit_parent_block_wins() {
    let mut context = media(15, MediaCategory::Safe, MediaRisk::Low);
    context.parent_blocked = true;
    assert_eq!(evaluate_media(&context), PolicyDecision::Block);
}

#[test]
fn unavailable_classifier_requires_parent() {
    let mut context = media(10, MediaCategory::Uncertain, MediaRisk::High);
    context.classifier_available = false;
    assert_eq!(evaluate_media(&context), PolicyDecision::RequireParent);
}

#[test]
fn young_child_adult_nudity_is_blocked() {
    let context = media(9, MediaCategory::AdultNudity, MediaRisk::High);
    assert_eq!(evaluate_media(&context), PolicyDecision::Block);
}

#[test]
fn young_child_sexualized_content_is_blocked() {
    let context = media(12, MediaCategory::SexualizedContent, MediaRisk::High);
    assert_eq!(evaluate_media(&context), PolicyDecision::Block);
}

#[test]
fn young_child_graphic_violence_is_blocked() {
    let context = media(7, MediaCategory::GraphicViolence, MediaRisk::High);
    assert_eq!(evaluate_media(&context), PolicyDecision::Block);
}

#[test]
fn high_risk_uncertain_media_requires_parent_for_any_age() {
    let context = media(16, MediaCategory::Uncertain, MediaRisk::High);
    assert_eq!(evaluate_media(&context), PolicyDecision::RequireParent);
}

#[test]
fn high_confidence_safe_media_is_allowed() {
    let context = media(11, MediaCategory::Safe, MediaRisk::Low);
    assert_eq!(evaluate_media(&context), PolicyDecision::Allow);
}

#[test]
fn teen_lower_risk_uncertain_requires_explicit_toggle() {
    let context = media(15, MediaCategory::Uncertain, MediaRisk::Medium);
    assert_eq!(evaluate_media(&context), PolicyDecision::RequireParent);

    let mut enabled = context;
    enabled.teen_uncertain_enabled = true;
    assert_eq!(evaluate_media(&enabled), PolicyDecision::Allow);
}

#[test]
fn low_confidence_unsafe_media_fails_closed() {
    let mut context = media(14, MediaCategory::AdultNudity, MediaRisk::High);
    context.high_confidence = false;
    assert_eq!(evaluate_media(&context), PolicyDecision::RequireParent);
}
