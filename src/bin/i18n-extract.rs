use regex::Regex;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Extract translation keys from Rust source files and generate a template YAML.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: i18n-extract <source_dir> [output_file]");
        eprintln!();
        eprintln!("  source_dir   Directory to scan for .rs files (default: ./src)");
        eprintln!("  output_file  Output YAML template (default: ./locales/template.yaml)");
        std::process::exit(1);
    }

    let source_dir = PathBuf::from(&args[1]);
    let output_file = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("locales/template.yaml")
    };

    if !source_dir.exists() {
        eprintln!("Error: source directory '{}' does not exist", source_dir.display());
        std::process::exit(1);
    }

    let mut keys = BTreeSet::new();

    // Collect all .rs files
    let mut rs_files = Vec::new();
    collect_rs_files(&source_dir, &mut rs_files);

    println!("Scanning {} Rust files...", rs_files.len());

    // Compile regexes
    // T::new("key")
    let re_new = Regex::new(r#"(?m)T::new\s*\(\s*"([^"]+)""#).unwrap();
    // T::with_vars("key", ...)
    let re_with_vars = Regex::new(r#"(?m)T::with_vars\s*\(\s*"([^"]+)""#).unwrap();
    // T::plural("key", ...)
    let re_plural = Regex::new(r#"(?m)T::plural\s*\(\s*"([^"]+)""#).unwrap();
    // T::with_context("key", "context")
    let re_context =
        Regex::new(r#"(?m)T::with_context\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)""#).unwrap();
    // T::ns("namespace").key("key")
    let re_ns =
        Regex::new(r#"(?m)T::ns\s*\(\s*"([^"]+)"\s*\)\s*\.\s*key\s*\(\s*"([^"]+)""#).unwrap();

    for file in &rs_files {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: could not read {}: {}", file.display(), e);
                continue;
            }
        };

        for caps in re_new.captures_iter(&content) {
            keys.insert(caps[1].to_string());
        }
        for caps in re_with_vars.captures_iter(&content) {
            keys.insert(caps[1].to_string());
        }
        for caps in re_plural.captures_iter(&content) {
            keys.insert(caps[1].to_string());
        }
        for caps in re_context.captures_iter(&content) {
            let key = &caps[1];
            let context = &caps[2];
            keys.insert(format!("{context}::{key}"));
        }
        for caps in re_ns.captures_iter(&content) {
            let namespace = &caps[1];
            let key = &caps[2];
            keys.insert(format!("{namespace}.{key}"));
        }
    }

    println!("Found {} unique keys", keys.len());

    // Generate YAML
    let mut yaml = String::from("# i18n template - auto-generated\n");
    yaml.push_str("# Copy this file and translate the values for each locale.\n\n");

    for key in &keys {
        yaml.push_str(&format!("{key}: \"\"\n"));
    }

    // Create parent directory if needed
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(&output_file, yaml).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {}", output_file.display(), e);
        std::process::exit(1);
    });

    println!("Template written to {}", output_file.display());
}

fn collect_rs_files(dir: &Path, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target directory
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                collect_rs_files(&path, results);
            } else if path.extension().is_some_and(|e| e == "rs") {
                results.push(path);
            }
        }
    }
}
