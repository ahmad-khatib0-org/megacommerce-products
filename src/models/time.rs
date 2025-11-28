use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use megacommerce_shared::models::translate::tr;
use serde_json::Value;

pub fn format_human_readable_time(lang: &str, timestamp_ms: i64, timezone_str: &str) -> String {
  let now_utc = Utc::now();
  let created_at_utc =
    DateTime::<Utc>::from_timestamp(timestamp_ms / 1000, (timestamp_ms % 1000) as u32 * 1_000_000)
      .unwrap_or_else(|| now_utc);

  // Parse the timezone string
  // Fallback to UTC if timezone is invalid
  let timezone: Tz = timezone_str.parse().unwrap_or_else(|_| chrono_tz::UTC);

  // Convert to user's local time
  let now = now_utc.with_timezone(&timezone);
  let created_at = created_at_utc.with_timezone(&timezone);

  let duration = now.signed_duration_since(created_at);

  // Handle minutes (less than 1 hour)
  if duration.num_minutes() < 60 {
    if duration.num_minutes() == 0 {
      return tr::<()>(lang, "time.just_now", None).unwrap_or("just now".to_string());
    } else if duration.num_minutes() == 1 {
      return tr::<()>(lang, "time.one_minute_ago", None).unwrap_or("one minute ago".to_string());
    } else {
      let params = Some(HashMap::from([(
        "Minutes".to_string(),
        Value::Number(duration.num_minutes().into()),
      )]));
      return tr(lang, "time.minutes_ago", params)
        .unwrap_or(format!("{} minutes ago", duration.num_minutes()));
    }
  }

  // Handle hours (1-23 hours)
  if duration.num_hours() < 24 {
    if duration.num_hours() == 1 {
      return tr::<()>(lang, "time.one_hour_ago", None).unwrap_or("one hour ago".to_string());
    } else {
      let params =
        Some(HashMap::from([("Hours".to_string(), Value::Number(duration.num_hours().into()))]));
      return tr(lang, "time.hours_ago", params)
        .unwrap_or(format!("{} hours ago", duration.num_hours()));
    }
  }

  // Handle days (1-6 days)
  if duration.num_days() < 7 {
    if duration.num_days() == 1 {
      return tr::<()>(lang, "time.yesterday", None).unwrap_or("yesterday".to_string());
    } else {
      let params =
        Some(HashMap::from([("Days".to_string(), Value::Number(duration.num_days().into()))]));
      return tr(lang, "time.days_ago", params)
        .unwrap_or(format!("{} days ago", duration.num_days()));
    }
  }

  // Handle weeks (1-4 weeks)
  if duration.num_weeks() < 5 {
    if duration.num_weeks() == 1 {
      return tr::<()>(lang, "time.one_week_ago", None).unwrap_or("one week ago".to_string());
    } else {
      let params =
        Some(HashMap::from([("Weeks".to_string(), Value::Number(duration.num_weeks().into()))]));
      return tr(lang, "time.weeks_ago", params)
        .unwrap_or(format!("{} weeks ago", duration.num_weeks()));
    }
  }

  // Handle months (more than 4 weeks)
  let months = duration.num_days() / 30;
  if months == 1 {
    return tr::<()>(lang, "time.one_month_ago", None).unwrap_or("one month ago".to_string());
  } else {
    let params = Some(HashMap::from([("Months".to_string(), Value::Number(months.into()))]));
    return tr(lang, "time.months_ago", params).unwrap_or(format!("{} months ago", months));
  }
}
