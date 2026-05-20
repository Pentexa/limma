/// Consent validator stub.
///
/// V1: Always grants consent. Real implementation will check DNS TXT records
/// and/or API key validation in a future phase.
pub struct ConsentValidatorStub;

impl ConsentValidatorStub {
    pub fn new() -> Self {
        Self
    }

    /// Stub: Always returns Ok (consent granted)
    pub async fn verify_consent(&self, _poc_type: &str) -> Result<(), String> {
        tracing::debug!("[ConsentValidatorStub] Consent verification stub — always granting");
        Ok(())
    }
}
