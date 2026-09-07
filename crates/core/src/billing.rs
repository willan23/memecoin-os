use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanId {
    Free,
    Pro,
    Research,
    Growth,
    Enterprise,
    Api,
}

impl PlanId {
    pub fn as_str(self) -> &'static str {
        match self {
            PlanId::Free => "free",
            PlanId::Pro => "pro",
            PlanId::Research => "research",
            PlanId::Growth => "growth",
            PlanId::Enterprise => "enterprise",
            PlanId::Api => "api",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "pro" => PlanId::Pro,
            "research" => PlanId::Research,
            "growth" => PlanId::Growth,
            "enterprise" => PlanId::Enterprise,
            "api" => PlanId::Api,
            _ => PlanId::Free,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanLimits {
    pub api_requests_day: i64,
    pub ai_requests_day: i64,
    pub research_day: i64,
    pub tokens_monitored: i64,
}

impl PlanId {
    pub fn limits(self) -> PlanLimits {
        match self {
            PlanId::Free => PlanLimits {
                api_requests_day: 1_000,
                ai_requests_day: 20,
                research_day: 5,
                tokens_monitored: 10,
            },
            PlanId::Pro => PlanLimits {
                api_requests_day: 20_000,
                ai_requests_day: 200,
                research_day: 50,
                tokens_monitored: 50,
            },
            PlanId::Research => PlanLimits {
                api_requests_day: 20_000,
                ai_requests_day: 500,
                research_day: 200,
                tokens_monitored: 50,
            },
            PlanId::Growth => PlanLimits {
                api_requests_day: 50_000,
                ai_requests_day: 500,
                research_day: 100,
                tokens_monitored: 200,
            },
            PlanId::Enterprise | PlanId::Api => PlanLimits {
                api_requests_day: -1,
                ai_requests_day: -1,
                research_day: -1,
                tokens_monitored: -1,
            },
        }
    }
}

/// -1 = unlimited. Over quota is a hard deny when enforce is on.
pub fn allowed(used: i64, limit: i64) -> bool {
    limit < 0 || used < limit
}

pub fn metric_for_path(path: &str) -> &'static str {
    if path.starts_with("/v1/ai/") || path.starts_with("/v2/intelligence") {
        "ai_requests"
    } else if path.starts_with("/v1/research") || path.starts_with("/v2/research") {
        "research_reports"
    } else {
        "api_requests"
    }
}

pub fn limit_for(plan: PlanId, metric: &str) -> i64 {
    let l = plan.limits();
    match metric {
        "ai_requests" => l.ai_requests_day,
        "research_reports" => l.research_day,
        "tokens_monitored" => l.tokens_monitored,
        _ => l.api_requests_day,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_caps_research_enterprise_unlimited() {
        assert!(!allowed(5, 5));
        assert!(allowed(4, 5));
        assert!(allowed(10_000, -1));
        assert_eq!(limit_for(PlanId::Free, "research_reports"), 5);
        assert_eq!(limit_for(PlanId::Enterprise, "api_requests"), -1);
    }

    #[test]
    fn research_path_meters_research_not_generic_api() {
        assert_eq!(metric_for_path("/v1/research"), "research_reports");
        assert_eq!(metric_for_path("/v1/tokens/pepe"), "api_requests");
    }
}
