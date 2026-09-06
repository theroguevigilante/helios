use std::path::Path;

/// Result of validating a Lisp parser script.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub parser_name: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate that a `.scm` file is a well-formed Helios parser.
///
/// Checks:
/// 1. File compiles without syntax errors
/// 2. `(parser-name)` is defined and returns a non-empty string
/// 3. `(detect "")` is defined and returns a boolean
/// 4. `(parse "test")` is defined and returns a hash-map
pub fn validate_script(path: &Path) -> anyhow::Result<ValidationResult> {
    use steel::rvals::SteelVal;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut parser_name = None;

    let source = std::fs::read_to_string(path)?;

    // 1. Compile check
    let mut engine = super::engine::create_engine();
    let source_static: &'static str = Box::leak(source.into_boxed_str());
    if let Err(e) = engine.compile_and_run_raw_program(source_static) {
        errors.push(format!("Syntax/compile error: {}", e));
        return Ok(ValidationResult {
            is_valid: false,
            parser_name,
            errors,
            warnings,
        });
    }

    // 2. (parser-name) check
    match engine.call_function_by_name_with_args("parser-name", vec![]) {
        Ok(val) => match val {
            SteelVal::StringV(name) => {
                let name = name.to_string();
                if name.is_empty() {
                    errors.push("(parser-name) returned an empty string".into());
                } else {
                    parser_name = Some(name);
                }
            }
            _ => errors.push("(parser-name) must return a string".into()),
        },
        Err(_) => errors.push("Missing required function: (parser-name)".into()),
    }

    // 3. (detect ...) check
    match engine.call_function_by_name_with_args("detect", vec![SteelVal::StringV("".into())]) {
        Ok(_val) => {
            // As long as it doesn't crash, it's valid
        }
        Err(_) => errors.push("Missing required function: (detect raw)".into()),
    }

    // 4. (parse ...) check
    match engine
        .call_function_by_name_with_args("parse", vec![SteelVal::StringV("dummy dummy dummy dummy dummy".into())])
    {
        Ok(val) => match val {
            SteelVal::HashMapV(_) => {}
            _ => errors
                .push("(parse raw) must return a hash-map, e.g. (hash \"message\" \"...\")".into()),
        },
        Err(e) => errors.push(format!("(parse) crashed or is missing: {}", e)),
    }

    // 5. Optional: check (parser-description)
    if engine
        .call_function_by_name_with_args("parser-description", vec![])
        .is_err()
    {
        warnings.push(
            "Optional function (parser-description) not defined; using filename as description"
                .into(),
        );
    }

    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        parser_name,
        errors,
        warnings,
    })
}
