use crate::infrastructure::active_detection::differential::BaselineProfile;

#[derive(Debug, Clone)]
pub struct DelayAnalysis {
    pub baseline_ms: Option<u64>,
    pub observed_ms: u64,
    pub expected_delay_ms: u64,
    pub delta_ms: Option<u64>,
    pub ratio: Option<f64>,
    pub significant: bool,
    pub confidence_score: f32,
}

impl DelayAnalysis {
    pub fn summary(&self) -> String {
        match (self.baseline_ms, self.delta_ms, self.ratio) {
            (Some(baseline), Some(delta), Some(ratio)) => format!(
                "Observed {}ms vs baseline {}ms (delta {}ms, ratio {:.2}x, expected injected delay {}ms)",
                self.observed_ms, baseline, delta, ratio, self.expected_delay_ms
            ),
            _ => format!(
                "Observed {}ms without baseline (expected injected delay {}ms)",
                self.observed_ms, self.expected_delay_ms
            ),
        }
    }
}

pub struct DelayAnalyzer;

impl DelayAnalyzer {
    pub fn analyze(
        baseline: Option<&BaselineProfile>,
        observed_ms: u64,
        expected_delay_ms: u64,
    ) -> DelayAnalysis {
        if let Some(baseline) = baseline {
            let baseline_ms = baseline
                .average_response_time_ms
                .max(baseline.response_time_ms);
            let delta_ms = observed_ms.saturating_sub(baseline_ms);
            let required_delta = (expected_delay_ms as f64 * 0.8).round() as u64;
            let required_delta = required_delta.max(4000);
            let significant = observed_ms > baseline_ms && delta_ms >= required_delta;
            let ratio = if baseline_ms > 0 {
                Some(observed_ms as f64 / baseline_ms as f64)
            } else {
                None
            };
            let confidence_score = if significant {
                let delta_score = (delta_ms as f32 / expected_delay_ms.max(1) as f32).min(1.0);
                let ratio_score = ratio
                    .map(|ratio| ((ratio as f32 - 1.0) / 3.0).clamp(0.0, 1.0))
                    .unwrap_or(0.5);
                0.65 + ((delta_score * 0.25) + (ratio_score * 0.10))
            } else {
                0.0
            };

            DelayAnalysis {
                baseline_ms: Some(baseline_ms),
                observed_ms,
                expected_delay_ms,
                delta_ms: Some(delta_ms),
                ratio,
                significant,
                confidence_score,
            }
        } else {
            let fallback_threshold = expected_delay_ms.saturating_sub(500).max(4500);
            let significant = observed_ms >= fallback_threshold;

            DelayAnalysis {
                baseline_ms: None,
                observed_ms,
                expected_delay_ms,
                delta_ms: None,
                ratio: None,
                significant,
                confidence_score: if significant { 0.55 } else { 0.0 },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(ms: u64) -> BaselineProfile {
        BaselineProfile {
            status_code: 200,
            content_length: 2,
            response_body: "ok".to_string(),
            response_time_ms: ms,
            average_response_time_ms: ms,
            body_hash: "hash".to_string(),
            header_fingerprint: Vec::new(),
            error_rate: 0.0,
            redirect_location: None,
        }
    }

    #[test]
    fn requires_delta_over_baseline() {
        let slow_baseline = baseline(4_000);
        let analysis = DelayAnalyzer::analyze(Some(&slow_baseline), 4_600, 5_000);

        assert!(!analysis.significant);
    }

    #[test]
    fn accepts_real_injected_delay() {
        let normal_baseline = baseline(250);
        let analysis = DelayAnalyzer::analyze(Some(&normal_baseline), 5_200, 5_000);

        assert!(analysis.significant);
    }
}
