use crate::domain::auth::AuthProfile;
use crate::domain::fuzzing::{EndpointContext, InsertionPoint};
use reqwest::{Client, RequestBuilder};
use url::Url;

pub struct RequestReplayer {
    client: Client,
    auth: Option<AuthProfile>,
    allowed_domains: Vec<String>,
    has_l3_consent: bool,
}

impl RequestReplayer {
    pub fn new(
        client: Client,
        auth: Option<AuthProfile>,
        allowed_domains: Vec<String>,
        has_l3_consent: bool,
    ) -> Self {
        Self {
            client,
            auth,
            allowed_domains,
            has_l3_consent,
        }
    }

    /// Builds a reqwest RequestBuilder based on the EndpointContext and the payload injection point.
    /// Includes safety checks (scope enforcement and L3 consent for destructive methods).
    pub fn build_request(
        &self,
        ctx: &EndpointContext,
        point: &InsertionPoint,
        payload: &str,
    ) -> Result<RequestBuilder, String> {
        // Scope enforcement
        if let Ok(parsed_url) = Url::parse(&ctx.url) {
            if let Some(host) = parsed_url.host_str() {
                let host = host.trim_end_matches('.').to_ascii_lowercase();
                if !self.allowed_domains.is_empty()
                    && !self.allowed_domains.iter().any(|domain| {
                        let domain = domain
                            .trim_start_matches('.')
                            .trim_end_matches('.')
                            .to_ascii_lowercase();
                        host == domain || host.ends_with(&format!(".{}", domain))
                    })
                {
                    return Err(format!("Out of scope URL: {}", ctx.url));
                }
            }
        }

        // L3 Consent check for destructive methods
        let method_upper = ctx.method.to_uppercase();
        if ["PUT", "PATCH", "DELETE"].contains(&method_upper.as_str()) && !self.has_l3_consent {
            return Err(format!(
                "Destructive method {} requires L3 consent",
                method_upper
            ));
        }

        let mut builder = match method_upper.as_str() {
            "POST" => self.client.post(&ctx.url),
            "PUT" => self.client.put(&ctx.url),
            "DELETE" => self.client.delete(&ctx.url),
            "PATCH" => self.client.patch(&ctx.url),
            _ => self.client.get(&ctx.url),
        };

        // Add original headers from EndpointContext
        for (k, v) in &ctx.headers {
            builder = builder.header(k, v);
        }

        // Apply AuthProfile headers if present (will overwrite original headers like Cookie if there's a conflict)
        if let Some(auth_profile) = &self.auth {
            for (k, v) in auth_profile.get_headers() {
                builder = builder.header(k, v);
            }
        }

        // Apply payload
        match point {
            InsertionPoint::QueryParam(param) => {
                builder = builder.query(&[(param, payload)]);
                // If it's a GET, body is usually empty, but we might still pass it just in case
                if let Some(ref b) = ctx.body {
                    if ctx.method != "GET" {
                        builder = builder.body(b.clone());
                    }
                }
            }
            InsertionPoint::JsonBodyPath(path) => {
                if let Some(ref b) = ctx.body {
                    if let Some(mutated_body) =
                        super::json_mutator::JsonMutator::inject_payload(b, path, payload)
                    {
                        builder = builder.header("Content-Type", "application/json");
                        builder = builder.body(mutated_body);
                    } else {
                        // Fallback to original body
                        builder = builder.body(b.clone());
                    }
                }
            }
            InsertionPoint::Header(header_name) => {
                // Remove original header if exists and replace with payload
                builder = builder.header(header_name, payload);
                if let Some(ref b) = ctx.body {
                    builder = builder.body(b.clone());
                }
            }
            InsertionPoint::FormData(field) => {
                // Simplified form-urlencoded support
                let form = vec![(field.as_str(), payload)];
                builder = builder.form(&form);
            }
        }

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_enforcement() {
        let client = Client::new();
        let replayer = RequestReplayer::new(client, None, vec!["example.com".to_string()], false);

        let mut ctx = EndpointContext::new("GET", "http://test.com/api");
        let point = InsertionPoint::QueryParam("q".to_string());

        // Out of scope
        let res = replayer.build_request(&ctx, &point, "payload");
        assert!(res.is_err());

        // In scope
        ctx.url = "http://api.example.com/api".to_string();
        let res2 = replayer.build_request(&ctx, &point, "payload");
        assert!(res2.is_ok());
    }

    #[test]
    fn test_l3_consent_enforcement() {
        let client = Client::new();
        let replayer = RequestReplayer::new(client, None, vec![], false);

        let ctx = EndpointContext::new("DELETE", "http://test.com/api");
        let point = InsertionPoint::QueryParam("q".to_string());

        // Destructive without consent
        let res = replayer.build_request(&ctx, &point, "payload");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("requires L3 consent"));

        // Destructive with consent
        let replayer_l3 = RequestReplayer::new(Client::new(), None, vec![], true);
        let res_l3 = replayer_l3.build_request(&ctx, &point, "payload");
        assert!(res_l3.is_ok());
    }
}
