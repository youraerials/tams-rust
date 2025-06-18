use crate::{error::TamsError, models::TimeRange};
use chrono::{DateTime, Utc};
use std::cmp::Ordering;

/// Parse a TAMS timestamp string in the format "seconds:nanoseconds"
/// where seconds is Unix timestamp and nanoseconds is the fractional part
pub fn parse_tams_timestamp(timestamp: &str) -> Result<DateTime<Utc>, TamsError> {
    let parts: Vec<&str> = timestamp.split(':').collect();
    
    if parts.len() != 2 {
        return Err(TamsError::InvalidTimerange(format!(
            "Invalid timestamp format: expected 'seconds:nanoseconds', got '{}'", 
            timestamp
        )));
    }
    
    let seconds: i64 = parts[0].parse()
        .map_err(|_| TamsError::InvalidTimerange(format!(
            "Invalid seconds value: '{}'", parts[0]
        )))?;
    
    let nanoseconds: u32 = parts[1].parse()
        .map_err(|_| TamsError::InvalidTimerange(format!(
            "Invalid nanoseconds value: '{}'", parts[1]
        )))?;
    
    // Validate nanoseconds range
    if nanoseconds >= 1_000_000_000 {
        return Err(TamsError::InvalidTimerange(format!(
            "Nanoseconds must be less than 1,000,000,000, got {}", nanoseconds
        )));
    }
    
    DateTime::from_timestamp(seconds, nanoseconds)
        .ok_or_else(|| TamsError::InvalidTimerange(format!(
            "Invalid timestamp: {}:{}", seconds, nanoseconds
        )))
}

/// Format a DateTime as a TAMS timestamp string
pub fn format_tams_timestamp(datetime: &DateTime<Utc>) -> String {
    format!("{}:{:09}", datetime.timestamp(), datetime.timestamp_subsec_nanos())
}

/// Compare two TAMS timestamps
pub fn compare_tams_timestamps(a: &str, b: &str) -> Result<Ordering, TamsError> {
    let dt_a = parse_tams_timestamp(a)?;
    let dt_b = parse_tams_timestamp(b)?;
    Ok(dt_a.cmp(&dt_b))
}

/// Validate a TimeRange
pub fn validate_timerange(timerange: &TimeRange) -> Result<(), TamsError> {
    // Parse start timestamp
    let start_dt = parse_tams_timestamp(&timerange.start)?;
    
    // Parse end timestamp (now always required)
    let end_dt = parse_tams_timestamp(&timerange.end)?;
    
    // End must be after start
    if end_dt <= start_dt {
        return Err(TamsError::InvalidTimerange(format!(
            "End timestamp ({}) must be after start timestamp ({})",
            timerange.end, timerange.start
        )));
    }
    
    Ok(())
}

/// Check if two TimeRanges overlap
pub fn timeranges_overlap(a: &TimeRange, b: &TimeRange) -> Result<bool, TamsError> {
    validate_timerange(a)?;
    validate_timerange(b)?;
    
    let a_start = parse_tams_timestamp(&a.start)?;
    let b_start = parse_tams_timestamp(&b.start)?;
    let a_end = parse_tams_timestamp(&a.end)?;
    let b_end = parse_tams_timestamp(&b.end)?;
    
    // Check for overlap - both ranges are now always bounded
    Ok(a_start < b_end && b_start < a_end)
}

/// Check if a timestamp falls within a TimeRange
pub fn timestamp_in_range(timestamp: &str, range: &TimeRange) -> Result<bool, TamsError> {
    validate_timerange(range)?;
    
    let ts = parse_tams_timestamp(timestamp)?;
    let range_start = parse_tams_timestamp(&range.start)?;
    let range_end = parse_tams_timestamp(&range.end)?;
    
    // Must be at or after start and before end (exclusive end)
    Ok(ts >= range_start && ts < range_end)
}

/// Create a TimeRange from start and end timestamps
pub fn create_timerange(start: &str, end: &str) -> Result<TimeRange, TamsError> {
    let timerange = TimeRange {
        start: start.to_string(),
        end: end.to_string(),
    };
    
    validate_timerange(&timerange)?;
    Ok(timerange)
}

/// Get the current time as a TAMS timestamp
pub fn current_tams_timestamp() -> String {
    format_tams_timestamp(&Utc::now())
}

/// Parse ISO 8601 timestamp and convert to TAMS format
pub fn iso8601_to_tams(iso_timestamp: &str) -> Result<String, TamsError> {
    let dt = DateTime::parse_from_rfc3339(iso_timestamp)
        .map_err(|e| TamsError::InvalidTimerange(format!(
            "Invalid ISO 8601 timestamp '{}': {}", iso_timestamp, e
        )))?;
    
    Ok(format_tams_timestamp(&dt.with_timezone(&Utc)))
}

/// Convert TAMS timestamp to ISO 8601 format
pub fn tams_to_iso8601(tams_timestamp: &str) -> Result<String, TamsError> {
    let dt = parse_tams_timestamp(tams_timestamp)?;
    Ok(dt.to_rfc3339())
}

/// Calculate duration between two TAMS timestamps in nanoseconds
pub fn calculate_duration_nanos(start: &str, end: &str) -> Result<i64, TamsError> {
    let start_dt = parse_tams_timestamp(start)?;
    let end_dt = parse_tams_timestamp(end)?;
    
    if end_dt <= start_dt {
        return Err(TamsError::InvalidTimerange(
            "End timestamp must be after start timestamp".to_string()
        ));
    }
    
    let duration = end_dt.signed_duration_since(start_dt);
    Ok(duration.num_nanoseconds().unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tams_timestamp() {
        // Valid timestamp
        let result = parse_tams_timestamp("1609459200:123456789");
        assert!(result.is_ok());
        
        // Invalid format
        assert!(parse_tams_timestamp("invalid").is_err());
        assert!(parse_tams_timestamp("1609459200").is_err());
        assert!(parse_tams_timestamp("1609459200:123456789:extra").is_err());
        
        // Invalid nanoseconds
        assert!(parse_tams_timestamp("1609459200:1000000000").is_err());
    }

    #[test]
    fn test_format_tams_timestamp() {
        let dt = DateTime::from_timestamp(1609459200, 123456789).unwrap();
        let formatted = format_tams_timestamp(&dt);
        assert_eq!(formatted, "1609459200:123456789");
    }

    #[test]
    fn test_timerange_validation() {
        // Valid range
        let valid_range = TimeRange {
            start: "1609459200:000000000".to_string(),
            end: "1609459260:000000000".to_string(),
        };
        assert!(validate_timerange(&valid_range).is_ok());
        
        // Invalid range (end before start)
        let invalid_range = TimeRange {
            start: "1609459260:000000000".to_string(),
            end: "1609459200:000000000".to_string(),
        };
        assert!(validate_timerange(&invalid_range).is_err());
    }

    #[test]
    fn test_timerange_overlap() {
        let range1 = TimeRange {
            start: "1609459200:000000000".to_string(),
            end: "1609459260:000000000".to_string(),
        };
        
        let range2 = TimeRange {
            start: "1609459230:000000000".to_string(),
            end: "1609459290:000000000".to_string(),
        };
        
        // These ranges should overlap
        assert!(timeranges_overlap(&range1, &range2).unwrap());
        
        let range3 = TimeRange {
            start: "1609459300:000000000".to_string(),
            end: "1609459360:000000000".to_string(),
        };
        
        // range1 and range3 should not overlap
        assert!(!timeranges_overlap(&range1, &range3).unwrap());
    }

    #[test]
    fn test_timestamp_in_range() {
        let range = TimeRange {
            start: "1609459200:000000000".to_string(),
            end: "1609459260:000000000".to_string(),
        };
        
        // Inside range
        assert!(timestamp_in_range("1609459230:000000000", &range).unwrap());
        
        // Before range
        assert!(!timestamp_in_range("1609459100:000000000", &range).unwrap());
        
        // After range
        assert!(!timestamp_in_range("1609459300:000000000", &range).unwrap());
    }

    #[test]
    fn test_iso8601_conversion() {
        let iso = "2021-01-01T00:00:00Z";
        let tams = iso8601_to_tams(iso).unwrap();
        let back_to_iso = tams_to_iso8601(&tams).unwrap();
        assert_eq!(iso, back_to_iso);
    }

    #[test]
    fn test_duration_calculation() {
        let start = "1609459200:000000000";
        let end = "1609459260:000000000";
        let duration = calculate_duration_nanos(start, end).unwrap();
        assert_eq!(duration, 60_000_000_000); // 60 seconds in nanoseconds
    }
} 