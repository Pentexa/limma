use serde_json::Value;

pub struct JsonMutator;

impl JsonMutator {
    /// Recursively injects a payload into a specified JSON pointer path.
    /// If `path` is empty or "/", it tries to replace the whole body if possible, 
    /// but usually we target specific keys.
    /// Format of path: `/user/email`
    pub fn inject_payload(body: &str, path: &str, payload: &str) -> Option<String> {
        let mut parsed: Value = serde_json::from_str(body).ok()?;
        
        if let Some(target) = parsed.pointer_mut(path) {
            *target = Value::String(payload.to_string());
            return serde_json::to_string(&parsed).ok();
        }
        
        None
    }

    /// Extracts all leaf string/number paths from a JSON body.
    /// Returns paths in JSON pointer format (e.g., "/user/email")
    pub fn extract_mutation_paths(body: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(parsed) = serde_json::from_str::<Value>(body) {
            Self::traverse(&parsed, String::new(), &mut paths);
        }
        paths
    }

    fn traverse(val: &Value, current_path: String, paths: &mut Vec<String>) {
        match val {
            Value::Object(map) => {
                for (k, v) in map {
                    let new_path = format!("{}/{}", current_path, k.replace("/", "~1"));
                    Self::traverse(v, new_path, paths);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_path = format!("{}/{}", current_path, i);
                    Self::traverse(v, new_path, paths);
                }
            }
            Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
                if !current_path.is_empty() {
                    paths.push(current_path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mutation_paths() {
        let body = r#"{"user": {"email": "test@test.com", "age": 30}, "items": ["a", "b"]}"#;
        let paths = JsonMutator::extract_mutation_paths(body);
        assert!(paths.contains(&"/user/email".to_string()));
        assert!(paths.contains(&"/user/age".to_string()));
        assert!(paths.contains(&"/items/0".to_string()));
        assert!(paths.contains(&"/items/1".to_string()));
    }

    #[test]
    fn test_inject_payload() {
        let body = r#"{"user": {"email": "test@test.com"}}"#;
        let mutated = JsonMutator::inject_payload(body, "/user/email", "<script>alert(1)</script>").unwrap();
        assert!(mutated.contains("<script>alert(1)</script>"));
        assert!(!mutated.contains("test@test.com"));
    }
}
