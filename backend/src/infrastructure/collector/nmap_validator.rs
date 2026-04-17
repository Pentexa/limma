use crate::domain::entities::{DecisionOutcome, DecisionTreeStep, PortProbeResult};

/// Cross-references native probe results against actual nmap truth logic
/// and applies a verification downgrade for any non-parity.
pub fn validate_parity(results: &mut [PortProbeResult], truth_open_ports: &[u16]) {
    for result in results.iter_mut() {
        if truth_open_ports.contains(&result.port) {
            continue; // Parity is good
        }

        // Port is NOT verified open by Nmap truth layer. 
        // We must downgrade any 'Verified' candidates to 'Suspected'
        for candidate in result.service_candidates.iter_mut() {
            if candidate.decision == DecisionOutcome::Verified {
                candidate.decision = DecisionOutcome::Suspected;
                candidate.verification_trail.push(DecisionTreeStep {
                    step: "Nmap Parity Check".to_string(),
                    detail: "Consistency failure. Nmap validation layer overriding status. Downgraded to Suspected.".to_string(),
                });
            }
        }
    }
}
