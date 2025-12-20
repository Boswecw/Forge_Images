//! Hashing System - SHA-256 for Manifests
//!
//! Provides deterministic, reproducible hashes for legal defensibility.

use sha2::{Sha256, Digest};
use serde::Serialize;
use serde_json::{Value, to_string};

/// Compute SHA-256 hash of bytes, return hex string
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Convert to canonical JSON (sorted keys, no whitespace)
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v: Value = serde_json::to_value(value)?;
    let sorted = sort_value(&v);
    to_string(&sorted)
}

fn sort_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            let sorted_map: serde_json::Map<String, Value> = sorted
                .into_iter()
                .map(|(k, v)| (k.clone(), sort_value(v)))
                .collect();
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(sort_value).collect())
        }
        _ => v.clone()
    }
}

/// Compute manifest hash for an asset
pub fn compute_manifest_hash<T: Serialize>(manifest: &T) -> Result<String, serde_json::Error> {
    let canonical = canonical_json(manifest)?;
    Ok(sha256_hex(canonical.as_bytes()))
}

/// Compute job hash for audit logging
/// job_hash = sha256(template_id + template_version + canonical_payload + engine_version)
pub fn compute_job_hash(
    template_id: &str,
    template_version: &str,
    payload: &impl Serialize,
    engine_version: &str,
) -> Result<String, serde_json::Error> {
    let canonical_payload = canonical_json(payload)?;
    let combined = format!(
        "{}:{}:{}:{}",
        template_id, template_version, canonical_payload, engine_version
    );
    Ok(sha256_hex(combined.as_bytes()))
}

// We need hex encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_json_sorted() {
        let obj = json!({"z": 1, "a": 2, "m": 3});
        let canonical = canonical_json(&obj).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"test data";
        let h1 = sha256_hex(data);
        let h2 = sha256_hex(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_manifest_hash_stable() {
        let manifest = json!({
            "template_id": "pwa-icon",
            "version": "1.0.0"
        });
        let h1 = compute_manifest_hash(&manifest).unwrap();
        let h2 = compute_manifest_hash(&manifest).unwrap();
        assert_eq!(h1, h2);
    }
}
