use std::time::Duration;

/// Format a `Duration` as ISO 8601 timestamp
#[allow(clippy::cast_possible_wrap)]
pub fn format_system_time(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // Simple ISO 8601 formatting without chrono dependency
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days_since_epoch as i64);

    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z"
    )
}

/// Convert days since epoch to (year, month, day)
/// Algorithm from <http://howardhinnant.github.io/date_algorithms.html>
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::missing_const_for_fn
)]
pub fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    #[allow(clippy::cast_lossless)]
    let year = yoe as i64 + era * 400;
    let day_of_year = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_system_time() {
        // Test epoch (1970-01-01 00:00:00)
        let duration = Duration::from_secs(0);
        assert_eq!(format_system_time(duration), "1970-01-01T00:00:00.000Z");

        // Test a known timestamp (2024-01-01 12:00:00)
        let duration = Duration::from_secs(1704110400);
        assert_eq!(format_system_time(duration), "2024-01-01T12:00:00.000Z");
    }

    #[test]
    fn test_days_to_date() {
        // Epoch
        assert_eq!(days_to_date(0), (1970, 1, 1));

        // Known dates
        assert_eq!(days_to_date(365), (1971, 1, 1));
        assert_eq!(days_to_date(19723), (2024, 1, 1));
    }
}
