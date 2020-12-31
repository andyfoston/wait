use std::collections::HashMap;
#[allow(unused_imports)]
use std::convert::TryFrom;
#[allow(unused_imports)]
use std::net::SocketAddr;
#[allow(unused_imports)]
use std::time::Duration;
#[allow(unused_imports)]
use super::{Config, DEFAULT_TIMEOUT, Target};

struct MockEnvConfig {
  env_vars: HashMap<String, String>,
}

impl MockEnvConfig {
  #[allow(dead_code)]
  fn new(env_vars: HashMap<String, String>) -> Self {
    Self {
      env_vars
    }
  }
}

impl Config for MockEnvConfig {
  fn vars(&self) -> Vec<(String, String)> {
    let mut results: Vec<(String, String)> = Vec::new();
    for (key, value) in self.env_vars.iter(){
      results.push((String::from(key), String::from(value)));
    }
    results
  }

  fn var(&self, key: &str) -> Result<String, String> {
    match self.env_vars.get(key) {
      Some(value) => Ok(value.clone()),
      None => Err(format!("Variable {} does not exist", &key))
    }
  }
}

#[test]
fn test_get_wait_target() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("WAIT_TARGETS".to_string(),
              "127.0.0.1:8000,127.0.0.1:8001".to_string());
  let config = MockEnvConfig::new(vars);
  let expected: Vec<Target> = vec![
    Target::try_from("127.0.0.1:8000").unwrap(),
    Target::try_from("127.0.0.1:8001").unwrap(),
  ];
  assert!(config.get_targets() == Ok(expected));
}

#[test]
fn test_get_target() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("TARGETS".to_string(),
              "127.0.0.1:8000,127.0.0.1:8001".to_string());
  let config = MockEnvConfig::new(vars);
  let expected: Vec<Target> = vec![
    Target::try_from("127.0.0.1:8000").unwrap(),
    Target::try_from("127.0.0.1:8001").unwrap(),
  ];
  assert!(config.get_targets() == Ok(expected));
}

#[test]
fn test_get_both_targets() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("WAIT_TARGETS".to_string(),
              "127.0.0.1:8000,127.0.0.1:8001".to_string());
  // The following should be ignored as WAIT_TARGETS should take presedence
  vars.insert("TARGETS".to_string(),
              "127.0.0.1:8002,127.0.0.1:8003".to_string());
  let config = MockEnvConfig::new(vars);
  let expected: Vec<Target> = vec![
    Target::try_from("127.0.0.1:8000").unwrap(),
    Target::try_from("127.0.0.1:8001").unwrap(),
  ];
  assert!(config.get_targets() == Ok(expected));
}

#[test]
fn test_get_linked_container_targets() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("DB_PORT_5432_TCP".to_string(),
              "tcp://172.17.0.5:5432".to_string());
  let config = MockEnvConfig::new(vars);
  let expected: Vec<Target> = vec![
    Target::try_from("172.17.0.5:5432").unwrap(),
  ];
  assert!(config.get_linked_container_targets() == Ok(expected));
}

#[test]
fn test_get_timeout_default() {
  let config = MockEnvConfig::new(HashMap::new());
  let expected = DEFAULT_TIMEOUT;
  assert!(config.get_timeout() == expected);
}

#[test]
fn test_get_timeout() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("WAIT_TIMEOUT".to_string(),
              "8".to_string());
  // The following should be ignored as WAIT_TARGETS should take presedence
  vars.insert("TIMEOUT".to_string(),
              "10".to_string());
  let config = MockEnvConfig::new(vars);
  let expected: Duration = Duration::from_secs(8);
  assert!(config.get_timeout() == expected);
}

#[test]
fn test_get_timeout_invalid() {
  let mut vars: HashMap<String, String> = HashMap::new();
  vars.insert("WAIT_TIMEOUT".to_string(),
              "invalid".to_string());
  let config = MockEnvConfig::new(vars);
  let expected = DEFAULT_TIMEOUT;
  assert!(config.get_timeout() == expected);
}

