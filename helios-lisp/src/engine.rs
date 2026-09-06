use super::builtins;
use steel::rvals::SteelVal;
use steel::steel_vm::engine::Engine;

/// Create a Steel engine pre-loaded with Helios builtins.
pub fn create_engine() -> Engine {
    let mut engine = Engine::new();
    builtins::register_all(&mut engine);
    engine
}

/// Convert a Steel hash-map value into a UniversalEvent.
pub fn steel_value_to_event(
    val: SteelVal,
    raw: &str,
) -> helios_core::Result<helios_core::UniversalEvent> {
    // The Lisp script returns a hash-map. We iterate over it to extract known keys.
    // Steel 0.8 HashMapV wraps an internal SteelHashMap. We convert to a Vec of pairs.

    // Try to extract the map as list of pairs
    let pairs = extract_hashmap_pairs(&val)
        .ok_or_else(|| helios_core::Error::ParseError("(parse) must return a hash-map".into()))?;

    let get_str = |key: &str| -> Option<String> {
        pairs.iter().find_map(|(k, v)| {
            if let SteelVal::StringV(k_str) = k {
                if k_str.as_str() == key {
                    if let SteelVal::StringV(v_str) = v {
                        return Some(v_str.to_string());
                    }
                }
            }
            None
        })
    };

    let timestamp = get_str("timestamp")
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .or_else(|_| chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ"))
                .ok()
        })
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    Ok(helios_core::UniversalEvent {
        timestamp,
        hostname: get_str("hostname"),
        service: get_str("service"),
        severity: get_str("severity"),
        message: get_str("message").unwrap_or_default(),
        raw_event: raw.to_string(),
        ..Default::default()
    })
}

/// Extract key-value pairs from a SteelVal that is a hash-map.
fn extract_hashmap_pairs(val: &SteelVal) -> Option<Vec<(SteelVal, SteelVal)>> {
    match val {
        SteelVal::HashMapV(hm) => {
            // SteelHashMap implements IntoIterator or we can convert to list
            Some(hm.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        }
        _ => None,
    }
}
