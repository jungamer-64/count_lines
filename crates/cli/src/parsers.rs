use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};
use std::{fmt::Display, str::FromStr};

/// Wrapper type to parse sizes with optional suffixes (e.g. 10K, 5MiB).
#[derive(Debug, Clone, Copy)]
pub struct SizeArg(pub u64);

impl std::str::FromStr for SizeArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().replace('_', "");
        let lower = s.to_ascii_lowercase();
        let (num_str, multiplier) = parse_with_suffix(&lower);
        let num: u64 = num_str
            .parse()
            .map_err(|_| format!("Invalid size number: {num_str}"))?;
        Ok(Self(num * multiplier))
    }
}

fn parse_with_suffix(s: &str) -> (&str, u64) {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    const SUFFIXES: &[(&[&str], u64)] = &[
        (&["tib", "tb", "t"], TB),
        (&["gib", "gb", "g"], GB),
        (&["mib", "mb", "m"], MB),
        (&["kib", "kb", "k"], KB),
    ];
    for (suffixes, multiplier) in SUFFIXES {
        for suffix in *suffixes {
            if let Some(stripped) = s.strip_suffix(suffix) {
                return (stripped.trim(), *multiplier);
            }
        }
    }
    (s, 1)
}

/// Wrapper type to parse date/time arguments in multiple formats.
#[derive(Debug, Clone, Copy)]
pub struct DateTimeArg(pub DateTime<Local>);

impl std::str::FromStr for DateTimeArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_rfc3339(s)
            .or_else(|| try_datetime_format(s))
            .or_else(|| try_date_format(s))
            .ok_or_else(|| format!("Cannot parse datetime: {s}"))
    }
}

fn try_rfc3339(s: &str) -> Option<DateTimeArg> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt: DateTime<FixedOffset>| DateTimeArg(dt.with_timezone(&Local)))
}

fn try_datetime_format(s: &str) -> Option<DateTimeArg> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|ndt| Local.from_local_datetime(&ndt).single())
        .map(DateTimeArg)
}

fn try_date_format(s: &str) -> Option<DateTimeArg> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .and_then(|nd: NaiveDate| nd.and_hms_opt(0, 0, 0))
        .and_then(|ndt| Local.from_local_datetime(&ndt).single())
        .map(DateTimeArg)
}

fn parse_bounded_number<T>(s: &str, min: T, max: Option<T>) -> Result<T, String>
where
    T: Copy + PartialOrd + Display + FromStr,
    <T as FromStr>::Err: Display,
{
    let value = s
        .parse::<T>()
        .map_err(|err| format!("invalid number '{s}': {err}"))?;
    if value < min {
        return Err(format!("value must be at least {min}"));
    }
    if let Some(max_bound) = max
        && value > max_bound
    {
        return Err(format!("value must be at most {max_bound}"));
    }
    Ok(value)
}

/// Parse a positive `usize` (>= 1) from CLI input.
///
/// # Errors
/// Returns an error if the input string is not a valid number or is less than 1.
pub fn parse_positive_usize(s: &str) -> Result<usize, String> {
    parse_bounded_number(s, 1, None)
}

/// Parse a `usize` constrained to the inclusive range [1, 512].
///
/// # Errors
/// Returns an error if the input string is not a valid number or is outside the range [1, 512].
pub fn parse_usize_1_to_512(s: &str) -> Result<usize, String> {
    parse_bounded_number(s, 1, Some(512))
}

/// Parse a positive `u64` (>= 1) from CLI input.
///
/// # Errors
/// Returns an error if the input string is not a valid number or is less than 1.
pub fn parse_positive_u64(s: &str) -> Result<u64, String> {
    parse_bounded_number(s, 1, None)
}

/// Parse a key=value pair string into a tuple.
///
/// # Errors
/// Returns an error if the input string does not contain an '=' character.
pub fn parse_key_val(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("Expected key=val: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_arg_basic() {
        let size: SizeArg = "1024".parse().unwrap();
        assert_eq!(size.0, 1024);
    }

    #[test]
    fn test_size_arg_with_suffix() {
        let size: SizeArg = "1K".parse().unwrap();
        assert_eq!(size.0, 1024);

        let size: SizeArg = "2M".parse().unwrap();
        assert_eq!(size.0, 2 * 1024 * 1024);

        let size: SizeArg = "1G".parse().unwrap();
        assert_eq!(size.0, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_size_arg_case_insensitive() {
        let size1: SizeArg = "1k".parse().unwrap();
        let size2: SizeArg = "1K".parse().unwrap();
        let size3: SizeArg = "1KB".parse().unwrap();
        let size4: SizeArg = "1KiB".parse().unwrap();
        assert_eq!(size1.0, size2.0);
        assert_eq!(size1.0, size3.0);
        assert_eq!(size1.0, size4.0);
    }

    #[test]
    fn test_parse_key_val() {
        let (k, v) = parse_key_val("foo=bar").unwrap();
        assert_eq!(k, "foo");
        assert_eq!(v, "bar");
    }

    #[test]
    fn test_parse_key_val_error() {
        assert!(parse_key_val("no_equals").is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Test that plain numeric values parse correctly without suffix
        #[test]
        fn test_size_arg_no_suffix(n in 0u64..1_000_000_000) {
            let formatted = format!("{n}");
            let parsed: SizeArg = formatted.parse().unwrap();
            prop_assert_eq!(parsed.0, n);
        }

        /// Test that K suffix correctly multiplies by 1024
        #[test]
        fn test_size_arg_k_suffix(n in 0u64..1_000_000) {
            let formatted = format!("{n}K");
            let parsed: SizeArg = formatted.parse().unwrap();
            prop_assert_eq!(parsed.0, n * 1024);
        }

        /// Test that M suffix correctly multiplies by 1024^2
        #[test]
        fn test_size_arg_m_suffix(n in 0u64..1_000) {
            let formatted = format!("{n}M");
            let parsed: SizeArg = formatted.parse().unwrap();
            prop_assert_eq!(parsed.0, n * 1024 * 1024);
        }

        /// Test that underscores are correctly ignored
        #[test]
        fn test_size_arg_underscores(n in 1000u64..1_000_000) {
            // Format with underscores as thousand separators
            let with_underscores = format!("{n}")
                .chars()
                .rev()
                .enumerate()
                .flat_map(|(i, c)| {
                    if i > 0 && i % 3 == 0 {
                        vec!['_', c]
                    } else {
                        vec![c]
                    }
                })
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();

            let parsed: SizeArg = with_underscores.parse().unwrap();
            prop_assert_eq!(parsed.0, n);
        }

        /// Test positive usize parsing
        #[test]
        fn test_positive_usize(n in 1usize..1_000_000) {
            let formatted = format!("{n}");
            let parsed = parse_positive_usize(&formatted).unwrap();
            prop_assert_eq!(parsed, n);
        }

        /// Test that zero is rejected for positive usize
        #[test]
        fn test_positive_usize_rejects_zero(_dummy in 0..1) {
            prop_assert!(parse_positive_usize("0").is_err());
        }

        /// Test bounded usize [1, 512]
        #[test]
        fn test_bounded_usize_valid(n in 1usize..=512) {
            let formatted = format!("{n}");
            let parsed = parse_usize_1_to_512(&formatted).unwrap();
            prop_assert_eq!(parsed, n);
        }

        /// Test bounded usize rejects values above max
        #[test]
        fn test_bounded_usize_rejects_large(n in 513usize..10_000) {
            let formatted = format!("{n}");
            prop_assert!(parse_usize_1_to_512(&formatted).is_err());
        }

        /// Test key=val parsing with arbitrary keys and values
        #[test]
        fn test_key_val_roundtrip(
            key in "[a-zA-Z][a-zA-Z0-9_]{0,20}",
            val in "[a-zA-Z0-9_]{0,50}"
        ) {
            let input = format!("{key}={val}");
            let (k, v) = parse_key_val(&input).unwrap();
            prop_assert_eq!(k, key);
            prop_assert_eq!(v, val);
        }
    }
}
