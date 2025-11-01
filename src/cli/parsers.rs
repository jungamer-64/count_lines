use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};

/// Wrapper type to parse sizes with optional suffixes (e.g. 10K, 5MiB).
#[derive(Debug, Clone, Copy)]
pub struct SizeArg(pub u64);

impl std::str::FromStr for SizeArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().replace('_', "");
        let lower = s.to_ascii_lowercase();
        let (num_str, multiplier) = parse_with_suffix(&lower)?;
        let num: u64 = num_str.parse().map_err(|_| format!("Invalid size number: {num_str}"))?;
        Ok(SizeArg(num * multiplier))
    }
}

fn parse_with_suffix(s: &str) -> Result<(&str, u64), String> {
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
                return Ok((stripped.trim(), *multiplier));
            }
        }
    }
    Ok((s, 1))
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
