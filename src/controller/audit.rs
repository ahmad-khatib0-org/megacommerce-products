use serde::Serialize;

pub(super) fn process_audit<T: Serialize>(_data: &T) {
  println!("called the process_audit");
}
