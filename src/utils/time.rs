use chrono::Local;

pub fn time_get_millis() -> u64 {
  Local::now().timestamp_millis().try_into().unwrap()
}
