use std::borrow::Cow;

/// Replace `{key}` placeholders in `template` with values from `vars`.
/// Unknown keys are left as-is (e.g. `{name}` stays `{name}`).
pub fn interpolate<'a>(template: &'a str, vars: &'a [(&'a str, &'a str)]) -> Cow<'a, str> {
    if vars.is_empty() || !template.contains('{') {
        return Cow::Borrowed(template);
    }

    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut changed = false;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut key = String::new();
            let mut found_close = false;
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == '}' {
                    found_close = true;
                    break;
                }
                key.push(next);
            }
            if found_close {
                if let Some((_, val)) = vars.iter().find(|(k, _)| *k == key) {
                    result.push_str(val);
                    changed = true;
                } else {
                    result.push('{');
                    result.push_str(&key);
                    result.push('}');
                }
            } else {
                result.push('{');
                result.push_str(&key);
            }
        } else {
            result.push(ch);
        }
    }

    if changed {
        Cow::Owned(result)
    } else {
        Cow::Borrowed(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_vars() {
        assert_eq!(interpolate("hello", &[]), "hello");
    }

    #[test]
    fn test_single_var() {
        assert_eq!(
            interpolate("Hello, {name}!", &[("name", "World")]),
            "Hello, World!"
        );
    }

    #[test]
    fn test_multiple_vars() {
        assert_eq!(
            interpolate("{a} + {b} = {c}", &[("a", "1"), ("b", "2"), ("c", "3")]),
            "1 + 2 = 3"
        );
    }

    #[test]
    fn test_unknown_var_passthrough() {
        assert_eq!(interpolate("Hi {name}", &[]), "Hi {name}");
    }
}
