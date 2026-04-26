use std::borrow::Cow;

/// Number formatting configuration.
///
/// Defines locale-specific rules for formatting numbers and currency
/// in translation templates via `{key::number}` and `{key::currency}` syntax.
///
/// # Example
/// ```
/// # use bevy_i18n::NumberFormat;
/// let us = NumberFormat {
///     thousands_sep: ',',
///     decimal_sep: '.',
///     decimal_places: Some(2),
///     currency_symbol: Some("$".to_string()),
/// };
/// assert_eq!(us.format_currency("1234.5"), "$ 1,234.50");
///
/// let eu = NumberFormat {
///     thousands_sep: '.',
///     decimal_sep: ',',
///     decimal_places: Some(2),
///     currency_symbol: Some("€".to_string()),
/// };
/// assert_eq!(eu.format_currency("1234.5"), "€ 1.234,50");
/// ```
#[derive(Clone, Debug)]
pub struct NumberFormat {
    /// Thousands separator (e.g. "," for en-US, "." for de-DE).
    pub thousands_sep: char,
    /// Decimal separator (e.g. "." for en-US, "," for de-DE).
    pub decimal_sep: char,
    /// Decimal places (None = auto-detect from input).
    pub decimal_places: Option<usize>,
    /// Currency symbol (e.g. "$", "€", "¥").
    pub currency_symbol: Option<String>,
}

impl NumberFormat {
    /// Default number format: no grouping, dot decimal separator.
    pub fn default_english() -> Self {
        Self {
            thousands_sep: ',',
            decimal_sep: '.',
            decimal_places: None,
            currency_symbol: None,
        }
    }

    /// Format a numeric string according to this locale's rules (no currency symbol).
    pub fn format_number(&self, value: &str) -> String {
        self.format_inner(value, false)
    }

    /// Format a numeric string with the currency symbol.
    pub fn format_currency(&self, value: &str) -> String {
        self.format_inner(value, true)
    }

    fn format_inner(&self, value: &str, include_currency: bool) -> String {
        // Parse the numeric value
        let parts: Vec<&str> = value.split('.').collect();
        let int_part = parts[0].trim_start_matches('-');
        let is_negative = value.starts_with('-');
        let frac_part = parts.get(1);

        // Group the integer part with thousands separator
        let grouped = self.group_int(int_part);

        // Handle decimal places
        let formatted_frac = match (frac_part, self.decimal_places) {
            (Some(frac), Some(places)) => {
                let padded = format!("{:0<width$}", frac, width = places);
                let trimmed = if places == 0 {
                    String::new()
                } else {
                    let s = &padded[..places.min(padded.len())];
                    // Remove trailing zeros only if decimal_places wasn't explicitly requested
                    if self.decimal_places.is_some() {
                        s.to_string()
                    } else {
                        s.trim_end_matches('0').to_string()
                    }
                };
                if trimmed.is_empty() {
                    String::new()
                } else {
                    format!("{}{}", self.decimal_sep, trimmed)
                }
            }
            (Some(frac), None) => format!("{}{}", self.decimal_sep, frac),
            (None, Some(0)) => String::new(),
            (None, Some(places)) => format!("{}{}", self.decimal_sep, "0".repeat(places)),
            (None, None) => String::new(),
        };

        let mut result = String::new();
        if is_negative {
            result.push('-');
        }
        if include_currency
            && let Some(ref sym) = self.currency_symbol
        {
            result.push_str(sym);
            result.push(' ');
        }
        result.push_str(&grouped);
        result.push_str(&formatted_frac);
        result
    }

    fn group_int(&self, int_part: &str) -> String {
        let len = int_part.len();
        if len <= 3 {
            return int_part.to_string();
        }

        let first_group_len = len % 3;
        let mut result = String::with_capacity(len + len / 3);

        if first_group_len > 0 {
            result.push_str(&int_part[..first_group_len]);
        }

        let mut start = first_group_len;
        while start < len {
            if !result.is_empty() {
                result.push(self.thousands_sep);
            }
            result.push_str(&int_part[start..start + 3]);
            start += 3;
        }

        result
    }
}

/// Format type recognized in `{key::format}` syntax.
#[derive(Debug, PartialEq)]
enum FormatType {
    Number,
    Currency,
}

impl FormatType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "number" => Some(FormatType::Number),
            "currency" => Some(FormatType::Currency),
            _ => None,
        }
    }
}

/// Replace `{key}` placeholders in `template` with values from `vars`.
/// Unknown keys are left as-is (e.g. `{name}` stays `{name}`).
///
/// This is a convenience wrapper around [`interpolate_with_format`] with no formatting.
#[allow(dead_code)]
pub fn interpolate<'a>(template: &'a str, vars: &'a [(&'a str, &'a str)]) -> Cow<'a, str> {
    interpolate_with_format(template, vars, None)
}

/// Replace `{key}` and `{key::format}` placeholders with formatting support.
/// When `num_format` is None, format specifiers are treated as plain keys.
///
/// Supported format types:
/// - `{key::number}` — format with thousands/decimal separators
/// - `{key::currency}` — format with currency symbol and separators
///
/// # Example
/// ```
/// # use bevy_i18n::NumberFormat;
/// # use bevy_i18n::interpolate::interpolate_with_format;
/// let fmt = NumberFormat {
///     thousands_sep: ',',
///     decimal_sep: '.',
///     decimal_places: None,
///     currency_symbol: Some("$".to_string()),
/// };
/// let result = interpolate_with_format(
///     "Total: {amount::currency}, Count: {n::number}",
///     &[("amount", "9999.99"), ("n", "12345")],
///     Some(&fmt),
/// );
/// assert_eq!(result, "Total: $ 9,999.99, Count: 12,345");
/// ```
pub fn interpolate_with_format<'a>(
    template: &'a str,
    vars: &'a [(&'a str, &'a str)],
    num_format: Option<&NumberFormat>,
) -> Cow<'a, str> {
    if vars.is_empty() || !template.contains('{') {
        return Cow::Borrowed(template);
    }

    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut changed = false;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut content = String::new();
            let mut found_close = false;
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == '}' {
                    found_close = true;
                    break;
                }
                content.push(next);
            }
            if found_close {
                // Check for format specifier: key::format
                let (key, format_type) = if let Some(pos) = content.find("::") {
                    let key = &content[..pos];
                    let fmt = &content[pos + 2..];
                    (key, FormatType::from_str(fmt))
                } else {
                    (content.as_str(), None)
                };

                if let Some((_, val)) = vars.iter().find(|(k, _)| *k == key) {
                    let formatted = match (&format_type, num_format) {
                        (Some(FormatType::Number), Some(fmt)) => fmt.format_number(val),
                        (Some(FormatType::Currency), Some(fmt)) => fmt.format_currency(val),
                        _ => val.to_string(),
                    };
                    result.push_str(&formatted);
                    changed = true;
                } else {
                    result.push('{');
                    result.push_str(&content);
                    result.push('}');
                }
            } else {
                result.push('{');
                result.push_str(&content);
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

    #[test]
    fn test_number_format_us() {
        let fmt = NumberFormat::default_english();
        assert_eq!(fmt.format_number("1234567.89"), "1,234,567.89");
        assert_eq!(fmt.format_number("1000"), "1,000");
        assert_eq!(fmt.format_number("999"), "999");
        assert_eq!(fmt.format_number("-5000"), "-5,000");
    }

    #[test]
    fn test_number_format_eu() {
        let fmt = NumberFormat {
            thousands_sep: '.',
            decimal_sep: ',',
            decimal_places: None,
            currency_symbol: None,
        };
        assert_eq!(fmt.format_number("1234567.89"), "1.234.567,89");
        assert_eq!(fmt.format_number("1000"), "1.000");
    }

    #[test]
    fn test_currency_format() {
        let fmt = NumberFormat {
            thousands_sep: ',',
            decimal_sep: '.',
            decimal_places: Some(2),
            currency_symbol: Some("$".to_string()),
        };
        assert_eq!(fmt.format_currency("1234.5"), "$ 1,234.50");
        assert_eq!(fmt.format_currency("100"), "$ 100.00");
    }

    #[test]
    fn test_interpolate_with_format() {
        let fmt = NumberFormat {
            thousands_sep: ',',
            decimal_sep: '.',
            decimal_places: None,
            currency_symbol: Some("$".to_string()),
        };
        let result = interpolate_with_format(
            "Price: {amount::currency}",
            &[("amount", "1234.5")],
            Some(&fmt),
        );
        assert_eq!(result, "Price: $ 1,234.5");
    }
}
