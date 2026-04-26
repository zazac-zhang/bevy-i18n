use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

/// Parsed locale data: key -> (raw value, variable set).
#[derive(Debug)]
struct LocaleData {
    /// All translation keys in this locale.
    keys: BTreeSet<String>,
    /// Key -> (raw template string, set of variable names used).
    entries: BTreeMap<String, (String, BTreeSet<String>)>,
}

/// Variable inconsistency detail for a single locale.
type VarDetail = Vec<(String, Vec<String>)>;

/// Result of a validation run.
#[derive(Debug)]
struct ValidationResult {
    missing_keys: BTreeMap<String, Vec<String>>,    // locale -> missing keys
    extra_keys: Vec<(String, String, Vec<String>)>, // (key, present_in, missing_from)
    var_inconsistencies: Vec<(String, VarDetail)>, // (key, [(locale, vars)])
}

impl ValidationResult {
    fn error_count(&self) -> usize {
        self.missing_keys.values().map(|v| v.len()).sum()
    }

    fn warning_count(&self) -> usize {
        self.extra_keys.len() + self.var_inconsistencies.len()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (locale_dir, template_file) = parse_args(&args);

    if !locale_dir.exists() {
        eprintln!("Error: locale directory '{}' does not exist", locale_dir.display());
        std::process::exit(1);
    }

    // Parse all YAML files once
    let mut locales: BTreeMap<String, LocaleData> = BTreeMap::new();
    let yaml_re = Regex::new(r#"^([^:#]+):\s*["']?(.*?)["']?(?:\s*#.*)?$"#).unwrap();
    let var_re = Regex::new(r"\{([^}:]+)(?:::[^}]+)?\}").unwrap();

    let mut yaml_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&locale_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                yaml_files.push(path);
            }
        }
    }

    if yaml_files.is_empty() {
        eprintln!("No YAML files found in {}", locale_dir.display());
        std::process::exit(1);
    }

    // Parse each locale file ONCE (fixes N+1 re-read)
    for file in &yaml_files {
        let locale = file.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(file).unwrap();

        let mut data = LocaleData {
            keys: BTreeSet::new(),
            entries: BTreeMap::new(),
        };

        for line in content.lines() {
            if let Some(caps) = yaml_re.captures(line.trim()) {
                let key = caps.get(1).unwrap().as_str().trim().to_string();
                let val = caps.get(2).unwrap().as_str().to_string();
                data.keys.insert(key.clone());

                let vars: BTreeSet<_> = var_re
                    .captures_iter(&val)
                    .filter_map(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .collect();
                data.entries.insert(key, (val, vars));
            }
        }

        locales.insert(locale, data);
    }

    println!("Validating {} locale files...", locales.len());

    // Load template keys if provided
    let template_keys = template_file.as_ref().and_then(|tf| {
        if tf.exists() {
            let content = fs::read_to_string(tf).ok()?;
            let mut keys = BTreeSet::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("variables:") {
                    continue;
                }
                if let Some(pos) = line.find(':') {
                    keys.insert(line[..pos].trim().to_string());
                }
            }
            Some(keys)
        } else {
            None
        }
    });

    let result = validate(&locales, template_keys.as_ref());
    print_report(&result);

    if result.error_count() > 0 {
        std::process::exit(1);
    }
}

fn parse_args(args: &[String]) -> (PathBuf, Option<PathBuf>) {
    let mut locale_dir = PathBuf::from("assets/locales");
    let mut template_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--template" | "-t" => {
                i += 1;
                if i < args.len() {
                    template_file = Some(PathBuf::from(&args[i]));
                }
            }
            "--dir" | "-d" => {
                i += 1;
                if i < args.len() {
                    locale_dir = PathBuf::from(&args[i]);
                }
            }
            other => {
                // Positional arg = locale dir
                if other.starts_with('-') {
                    eprintln!("Unknown flag: {}", other);
                    print_usage();
                    std::process::exit(1);
                }
                locale_dir = PathBuf::from(other);
            }
        }
        i += 1;
    }

    (locale_dir, template_file)
}

fn print_usage() {
    eprintln!("Usage: i18n-validate [OPTIONS] [locale_dir]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -t, --template <file>  Compare against a template YAML file");
    eprintln!("  -d, --dir <dir>        Locale directory (default: assets/locales)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  i18n-validate assets/locales");
    eprintln!("  i18n-validate -t locales/template.yaml assets/locales");
}

fn validate(
    locales: &BTreeMap<String, LocaleData>,
    template: Option<&BTreeSet<String>>,
) -> ValidationResult {
    let mut result = ValidationResult {
        missing_keys: BTreeMap::new(),
        extra_keys: Vec::new(),
        var_inconsistencies: Vec::new(),
    };

    // All unique keys across all locales + template
    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    for data in locales.values() {
        all_keys.extend(data.keys.iter().cloned());
    }
    if let Some(tmpl) = template {
        all_keys.extend(tmpl.iter().cloned());
    }

    let num_locales = locales.len();
    let locale_names: Vec<String> = locales.keys().cloned().collect();

    // Check 1: Missing keys per locale (including template keys if provided)
    for (locale, data) in locales {
        let missing: Vec<String> = all_keys
            .iter()
            .filter(|k| !data.keys.contains(*k))
            .cloned()
            .collect();
        if !missing.is_empty() {
            result.missing_keys.insert(locale.clone(), missing);
        }
    }

    // Check 2: Extra keys (present in some but not all locales)
    let reference_keys: BTreeSet<String> = if let Some(tmpl) = template {
        tmpl.iter().cloned().collect()
    } else {
        all_keys.clone()
    };

    for key in &all_keys {
        // Skip template-only keys for "extra" check
        if template.is_some() && !reference_keys.contains(key) {
            continue;
        }

        let present_in: Vec<String> = locales
            .iter()
            .filter(|(_, data)| data.keys.contains(key))
            .map(|(locale, _)| locale.clone())
            .collect();

        if present_in.len() < num_locales && !present_in.is_empty() {
            let missing_from: Vec<String> = locale_names
                .iter()
                .filter(|l| !present_in.contains(l))
                .cloned()
                .collect();
            result.extra_keys.push((
                key.clone(),
                present_in.join(", "),
                missing_from,
            ));
        }
    }

    // Check 3: Variable consistency (single pass over already-parsed data)
    let mut key_vars: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for (locale, data) in locales {
        for (key, (_val, vars)) in &data.entries {
            let var_str = vars.iter().cloned().collect::<Vec<_>>().join(", ");
            key_vars
                .entry(key.clone())
                .or_default()
                .entry(locale.clone())
                .or_default()
                .push(var_str);
        }
    }

    for (key, locale_vars) in &key_vars {
        let unique_vars: BTreeSet<_> = locale_vars.values().flatten().collect();
        if unique_vars.len() > 1 {
            let detail: Vec<(String, Vec<String>)> = locale_vars
                .iter()
                .map(|(locale, vars)| (locale.clone(), vars.clone()))
                .collect();
            result.var_inconsistencies.push((key.clone(), detail));
        }
    }

    result
}

fn print_report(result: &ValidationResult) {
    // Missing keys
    println!("\n=== Missing Keys ===");
    if result.missing_keys.is_empty() {
        println!("  None - all locales have all keys");
    } else {
        for (locale, keys) in &result.missing_keys {
            println!("  [{locale}] Missing {} key(s):", keys.len());
            for key in keys {
                println!("    - {key}");
            }
        }
    }

    // Extra keys
    println!("\n=== Extra Keys ===");
    if result.extra_keys.is_empty() {
        println!("  None - all keys consistent across locales");
    } else {
        for (key, present_in, missing_from) in &result.extra_keys {
            println!(
                "  '{key}' present in [{present_in}] but missing from [{missing_from:?}]"
            );
        }
    }

    // Variable inconsistencies
    println!("\n=== Variable Inconsistencies ===");
    if result.var_inconsistencies.is_empty() {
        println!("  None - all variables consistent");
    } else {
        for (key, detail) in &result.var_inconsistencies {
            println!("  '{key}':");
            for (locale, vars) in detail {
                println!("    {locale}: {{{}}}", vars.join("} {"));
            }
        }
    }

    println!();
    let errors = result.error_count();
    let warnings = result.warning_count();
    if errors > 0 || warnings > 0 {
        println!("Result: {errors} errors, {warnings} warnings");
    } else {
        println!("Result: All keys consistent across all locales");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_locale(name: &str, entries: &[(&str, &str)]) -> (String, LocaleData) {
        let mut data = LocaleData {
            keys: BTreeSet::new(),
            entries: BTreeMap::new(),
        };
        let var_re = Regex::new(r"\{([^}:]+)(?:::[^}]+)?\}").unwrap();
        for (key, val) in entries {
            data.keys.insert(key.to_string());
            let vars: BTreeSet<_> = var_re
                .captures_iter(val)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .collect();
            data.entries.insert(key.to_string(), (val.to_string(), vars));
        }
        (name.to_string(), data)
    }

    #[test]
    fn test_consistent_locales() {
        let locales: BTreeMap<_, _> = [
            make_locale("en", &[("greeting", "Hello {name}"), ("bye", "Goodbye")]),
            make_locale("zh", &[("greeting", "你好 {name}"), ("bye", "再见")]),
        ]
        .into_iter()
        .collect();

        let result = validate(&locales, None);
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_missing_keys() {
        let locales: BTreeMap<_, _> = [
            make_locale("en", &[("greeting", "Hello"), ("bye", "Goodbye")]),
            make_locale("zh", &[("greeting", "你好")]),
        ]
        .into_iter()
        .collect();

        let result = validate(&locales, None);
        assert_eq!(result.error_count(), 1);
        assert!(result.missing_keys["zh"].contains(&"bye".to_string()));
    }

    #[test]
    fn test_variable_inconsistency() {
        let locales: BTreeMap<_, _> = [
            make_locale("en", &[("msg", "Hello {name}, you have {count} items")]),
            make_locale("zh", &[("msg", "你好 {name}")]),
        ]
        .into_iter()
        .collect();

        let result = validate(&locales, None);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(result.var_inconsistencies.len(), 1);
        assert_eq!(result.var_inconsistencies[0].0, "msg");
    }

    #[test]
    fn test_template_comparison() {
        let locales: BTreeMap<_, _> = [
            make_locale("en", &[("key1", "v1"), ("key2", "v2")]),
        ]
        .into_iter()
        .collect();

        let mut template = BTreeSet::new();
        template.insert("key1".to_string());
        template.insert("key2".to_string());
        template.insert("key3".to_string()); // missing from en

        let result = validate(&locales, Some(&template));
        assert_eq!(result.error_count(), 1);
        assert!(result.missing_keys["en"].contains(&"key3".to_string()));
    }
}
