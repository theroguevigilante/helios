use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;

/// Register all Helios built-in functions into the Steel engine.
pub fn register_all(engine: &mut Engine) {
    // String utilities
    engine.register_fn("string-contains?", string_contains);
    engine.register_fn("string-starts-with?", string_starts_with);
    engine.register_fn("string-ends-with?", string_ends_with);
    engine.register_fn("helios/split", string_split);
    engine.register_fn("string-trim", string_trim);
    engine.register_fn("string-upper", string_upper);
    engine.register_fn("string-lower", string_lower);
    engine.register_fn("helios/substring", substring);
    engine.register_fn("string-length", string_length);
    engine.register_fn("string-replace", string_replace);

    // Regex
    engine.register_fn("regex-match?", regex_match);
    engine.register_fn("regex-captures", regex_captures);
    engine.register_fn("regex-find-first", regex_find_first);

    // Timestamp helpers
    engine.register_fn("parse-rfc3339", parse_rfc3339);
    engine.register_fn("now-utc", now_utc);
}

fn string_contains(haystack: String, needle: String) -> bool {
    haystack.contains(&needle)
}

fn string_starts_with(s: String, prefix: String) -> bool {
    s.starts_with(&prefix)
}

fn string_ends_with(s: String, suffix: String) -> bool {
    s.ends_with(&suffix)
}

fn string_split(s: String, delim: String) -> Vec<String> {
    s.split(&delim).map(|x| x.to_string()).collect()
}

fn string_trim(s: String) -> String {
    s.trim().to_string()
}

fn string_upper(s: String) -> String {
    s.to_uppercase()
}

fn string_lower(s: String) -> String {
    s.to_lowercase()
}

fn string_length(s: String) -> usize {
    s.len()
}

fn string_replace(s: String, from: String, to: String) -> String {
    s.replace(&from, &to)
}

fn substring(s: String, start: usize, end: usize) -> String {
    s.get(start..end).unwrap_or("").to_string()
}

fn regex_match(pattern: String, text: String) -> bool {
    regex::Regex::new(&pattern)
        .map(|re| re.is_match(&text))
        .unwrap_or(false)
}

fn regex_captures(pattern: String, text: String) -> Vec<String> {
    regex::Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(&text))
        .map(|caps| {
            caps.iter()
                .skip(1) // skip full match
                .filter_map(|m| m.map(|m| m.as_str().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn regex_find_first(pattern: String, text: String) -> String {
    regex::Regex::new(&pattern)
        .ok()
        .and_then(|re| re.find(&text))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}

fn parse_rfc3339(s: String) -> String {
    chrono::DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339())
}

fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}
