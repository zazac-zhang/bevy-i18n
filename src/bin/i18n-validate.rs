use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let locale_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("assets/locales")
    };

    if !locale_dir.exists() {
        eprintln!("Error: locale directory '{}' does not exist", locale_dir.display());
        std::process::exit(1);
    }

    let mut locales: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let yaml_re = Regex::new(r#"^([^:#]+):\s*["']?(.*?)["']?(?:\s*#.*)?$"#).unwrap();

    // Collect YAML files
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

    // Parse each locale file
    for file in &yaml_files {
        let locale = file
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let content = fs::read_to_string(file).unwrap();

        let mut keys = Vec::new();
        for line in content.lines() {
            if let Some(caps) = yaml_re.captures(line.trim()) {
                let key = caps.get(1).unwrap().as_str().trim().to_string();
                keys.push(key);
            }
        }

        locales.insert(locale, keys);
    }

    println!("Validating {} locale files...", locales.len());

    let mut all_keys: BTreeSet<&String> = BTreeSet::new();
    for keys in locales.values() {
        for key in keys {
            all_keys.insert(key);
        }
    }

    let mut errors = 0;
    let mut warnings = 0;

    // Check 1: Missing keys in each locale
    println!("\n=== Missing Keys ===");
    for (locale, keys) in &locales {
        let key_set: BTreeSet<&String> = keys.iter().collect();
        let missing: Vec<_> = all_keys.iter().filter(|k| !key_set.contains(*k)).collect();
        if !missing.is_empty() {
            errors += missing.len();
            for key in missing {
                println!("  [{locale}] Missing: {key}");
            }
        } else {
            println!("  [{locale}] OK ({keys_len} keys)", keys_len = keys.len());
        }
    }

    // Check 2: Extra keys (keys in one locale but not others)
    println!("\n=== Extra Keys (present in some but not all) ===");
    let num_locales = locales.len();
    for key in &all_keys {
        let locale_keys: Vec<&String> = locales.keys().collect();
        let in_locales: Vec<&String> = locales
            .iter()
            .filter(|(_, keys)| keys.iter().any(|k| k == *key))
            .map(|(locale, _)| locale)
            .collect();
        if !in_locales.is_empty() && in_locales.len() < num_locales {
            warnings += 1;
            let missing_from: Vec<_> = locale_keys.iter().filter(|l| !in_locales.contains(l)).map(|s| s.as_str()).collect::<Vec<_>>();
            let in_locales_str: Vec<_> = in_locales.iter().map(|s| s.as_str()).collect();
            println!(
                "  '{key}' in {} but missing from {}",
                in_locales_str.join(", "),
                missing_from.join(", ")
            );
        }
    }

    // Check 3: Variable consistency
    let var_re = Regex::new(r"\{([^}:]+)(?:::[^}]+)?\}").unwrap();

    println!("\n=== Variable Inconsistencies ===");
    // Group values by key across locales
    let mut key_values: BTreeMap<&String, BTreeMap<&String, Vec<String>>> = BTreeMap::new();

    for (locale, keys) in &locales {
        for key in keys {
            // We need the actual values - re-parse with values
            let content = fs::read_to_string(
                locale_dir.join(format!("{locale}.yaml")),
            )
            .unwrap_or_default();

            for line in content.lines() {
                if let Some(caps) = yaml_re.captures(line.trim()) {
                    let k = caps.get(1).unwrap().as_str().trim();
                    if k == key.as_str() {
                        let val = caps.get(2).unwrap().as_str().trim().to_string();
                        let vars: BTreeSet<_> = var_re
                            .captures_iter(&val)
                            .filter_map(|c| c.get(1))
                            .map(|m| m.as_str().to_string())
                            .collect();
                        let var_str: String = vars
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        key_values
                            .entry(key)
                            .or_default()
                            .entry(locale)
                            .or_default()
                            .push(var_str);
                    }
                }
            }
        }
    }

    for (key, locale_vars) in &key_values {
        let unique_vars: BTreeSet<_> = locale_vars.values().flatten().collect();
        if unique_vars.len() > 1 {
            warnings += 1;
            println!("  '{key}':");
            for (locale, vars) in locale_vars {
                println!("    {locale}: {{{vars}}}", vars = vars.join("} {"));
            }
        }
    }

    println!();
    if errors > 0 || warnings > 0 {
        println!("Result: {errors} errors, {warnings} warnings");
        if errors > 0 {
            std::process::exit(1);
        }
        std::process::exit(0);
    } else {
        println!("Result: All keys consistent across all locales");
        std::process::exit(0);
    }
}
