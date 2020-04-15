use super::*;
use std::time::Duration;

#[test]
fn validate_parse_interval() {
    assert_eq!(Duration::from_secs(678), parse_interval("678").unwrap());
}
