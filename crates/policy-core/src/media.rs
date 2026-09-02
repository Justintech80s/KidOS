use crate::PolicyDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaCategory {
    Safe,
    AdultNudity,
    SexualizedContent,
    GraphicViolence,
    SelfHarm,
    Drugs,
    ExtremistContent,
    Scam,
    Uncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaContext {
    pub age: u8,
    pub category: MediaCategory,
    pub risk: MediaRisk,
    pub high_confidence: bool,
    pub parent_blocked: bool,
    pub classifier_available: bool,
    pub teen_uncertain_enabled: bool,
}

pub fn evaluate_media(context: &MediaContext) -> PolicyDecision {
    if context.parent_blocked {
        return PolicyDecision::Block;
    }

    if !context.classifier_available {
        return PolicyDecision::RequireParent;
    }

    if !context.high_confidence && context.category != MediaCategory::Safe {
        return PolicyDecision::RequireParent;
    }

    if context.category == MediaCategory::Uncertain {
        if context.risk == MediaRisk::High {
            return PolicyDecision::RequireParent;
        }

        if context.age >= 13 && context.teen_uncertain_enabled {
            return PolicyDecision::Allow;
        }

        return PolicyDecision::RequireParent;
    }

    if context.age <= 12
        && context.high_confidence
        && matches!(
            context.category,
            MediaCategory::AdultNudity
                | MediaCategory::SexualizedContent
                | MediaCategory::GraphicViolence
        )
    {
        return PolicyDecision::Block;
    }

    if context.category == MediaCategory::Safe && context.high_confidence {
        return PolicyDecision::Allow;
    }

    PolicyDecision::RequireParent
}
