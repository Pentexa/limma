use crate::infrastructure::active_detection::differential::BaselineProfile;

#[derive(Debug, Clone)]
pub struct ResponseSnapshot {
    pub status_code: u16,
    pub content_length: usize,
    pub body_hash: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ResponseDiffResult {
    pub status_changed: bool,
    pub length_delta_ratio: f64,
    pub similarity: f64,
    pub baseline_hash: String,
    pub observed_hash: String,
}

impl ResponseDiffResult {
    pub fn is_significant(&self) -> bool {
        self.status_changed || self.length_delta_ratio > 0.05 || self.similarity < 0.92
    }

    pub fn summary(&self) -> String {
        format!(
            "status_changed={}, length_delta={:.2}%, similarity={:.2}, baseline_hash={}, observed_hash={}",
            self.status_changed,
            self.length_delta_ratio * 100.0,
            self.similarity,
            self.baseline_hash,
            self.observed_hash
        )
    }
}

pub struct ResponseDiffAnalyzer;

impl ResponseDiffAnalyzer {
    pub fn snapshot(status_code: u16, body: impl Into<String>) -> ResponseSnapshot {
        let body = body.into();
        ResponseSnapshot {
            status_code,
            content_length: body.len(),
            body_hash: stable_hash(&body),
            body,
        }
    }

    pub fn compare_snapshots(
        baseline: &ResponseSnapshot,
        observed: &ResponseSnapshot,
    ) -> ResponseDiffResult {
        let length_delta_ratio = if baseline.content_length > 0 {
            (baseline.content_length as f64 - observed.content_length as f64).abs()
                / baseline.content_length as f64
        } else if observed.content_length > 0 {
            1.0
        } else {
            0.0
        };

        ResponseDiffResult {
            status_changed: baseline.status_code != observed.status_code,
            length_delta_ratio,
            similarity: body_similarity(&baseline.body, &observed.body),
            baseline_hash: baseline.body_hash.clone(),
            observed_hash: observed.body_hash.clone(),
        }
    }

    pub fn compare_to_baseline(
        baseline: &BaselineProfile,
        observed_status: u16,
        observed_body: &str,
    ) -> ResponseDiffResult {
        let baseline_snapshot = ResponseSnapshot {
            status_code: baseline.status_code,
            content_length: baseline.content_length,
            body_hash: baseline.body_hash.clone(),
            body: baseline.response_body.clone(),
        };
        let observed_snapshot = Self::snapshot(observed_status, observed_body.to_string());

        Self::compare_snapshots(&baseline_snapshot, &observed_snapshot)
    }
}

pub fn stable_hash(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn body_similarity(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }

    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    let compared_len = left_bytes.len().min(right_bytes.len()).min(8192);
    if compared_len == 0 {
        return 0.0;
    }

    let equal_positions = left_bytes
        .iter()
        .zip(right_bytes.iter())
        .take(compared_len)
        .filter(|(left, right)| left == right)
        .count();
    let positional_score = equal_positions as f64 / compared_len as f64;
    let length_score = left_bytes.len().min(right_bytes.len()) as f64
        / left_bytes.len().max(right_bytes.len()) as f64;

    (positional_score * 0.7) + (length_score * 0.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_significant_length_delta() {
        let base = ResponseDiffAnalyzer::snapshot(200, "abcdef");
        let observed = ResponseDiffAnalyzer::snapshot(200, "abcdef-extra-content");
        let diff = ResponseDiffAnalyzer::compare_snapshots(&base, &observed);

        assert!(diff.is_significant());
    }
}
