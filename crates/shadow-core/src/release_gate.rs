use crate::score::ShadowScore;

#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseDecision {
    Approved,
    Blocked,
}

pub fn evaluate_release(score: &ShadowScore) -> ReleaseDecision {
    if score.critical > 0 {
        return ReleaseDecision::Blocked;
    }

    if score.percentage < 98.0 {
        return ReleaseDecision::Blocked;
    }

    ReleaseDecision::Approved
}
