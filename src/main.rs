use std::convert::TryFrom;
use std::env;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::time::{Duration, Instant};
use std::thread::sleep;

mod tests;

static DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(PartialEq)]
struct Target {
  target_name: String,
  addr: SocketAddr,
}

impl TryFrom<&str> for Target {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value.to_socket_addrs() {
      Ok(mut socket_addrs) => {
        match socket_addrs.next() {
          Some(addr) => return Ok(Self{target_name: value.to_string(), addr: addr}),
          None => return Err(format!("Failed to parse: {}", value)),
        }
      },
      Err(_) => Err(format!("Failed to parse: {}", value)),
    }
  }
}

impl Target {
  fn poll(&self, timeout: &Duration) -> Result<bool, String> {
    let start = Instant::now();
    let sleep_duration = Duration::new(1, 0);
    let mut found = false;
    print!("Connecting to {} ", &self.target_name);
    let _ = io::stdout().flush();
    while start.elapsed() <= *timeout {
      match TcpStream::connect_timeout(&self.addr, *timeout) {
        Ok(_) => {
          println!(" Connected!");
          let _ = io::stdout().flush();
          found = true;
          break;
        },
        Err(_) => {
          print!(".");
          let _ = io::stdout().flush();
          sleep(sleep_duration);
        }
      }
    }
    if !found {
      println!(" failed to connect!");
      Err(format!("Failed to connect to: {}", &self.target_name))
    } else {
      Ok(true)
    }

  }
}

struct EnvConfig {}

trait Config {
  fn vars(&self) -> Vec<(String, String)> ;
  fn var(&self, key: &str) -> Result<String, String>;

  fn get_targets(&self) -> Result<Vec<Target>, String>{
    let mut results: Vec<Target> = Vec::new();

    let env_names: [&str; 2] = ["WAIT_TARGETS", "TARGETS"];
    for env_name in env_names.iter() {
      if let Ok(targets) = self.var(env_name) {
        for target in targets.split(",") {
          match Target::try_from(target) {
            Ok(addr) => results.push(addr),
            Err(e) => return Err(e)
          }
        }
      }
      if !results.is_empty() {
        return Ok(results);
      }
    }
    Ok(results)
  }

  fn get_linked_container_targets(&self) -> Result<Vec<Target>, String> {
    let mut results: Vec<Target> = Vec::new();
    for (key, value) in self.vars().iter() {
      if key.ends_with("_TCP") {
        match Target::try_from(&value[6..]) {
          Ok(addr) => results.push(addr),
          Err(e) => return Err(format!("Failed to parse: {}", e)),
        }
      }
    }
    Ok(results)
  }

  fn get_timeout(&self) -> Duration {
    let timeout_var_names: [&str; 2] = ["WAIT_TIMEOUT", "TIMEOUT"];
    for var_name in timeout_var_names.iter() {
      if let Ok(val) = self.var(var_name) {
        match val.parse::<u64>() {
          Ok(res) => return Duration::from_secs(res),
          Err(_) => {
            println!("Failed to parse timeout of {}. Using the default {} second timeout", val, DEFAULT_TIMEOUT.as_secs());
            return DEFAULT_TIMEOUT;
          }
        } 
      } 
    }
    DEFAULT_TIMEOUT
  }

  fn get_all_targets(&self) -> Result<Vec<Target>, String> {
    let mut results: Vec<Target> = self.get_targets()?;
    if !results.is_empty() {
      return Ok(results);
    }

    results = self.get_linked_container_targets()?; 
    if !results.is_empty() {
      return Ok(results);
    }

    Err(String::from("Cannot find any targets to poll"))
  }
}

impl Config for EnvConfig {
  fn vars(&self) -> Vec<(String, String)> {
    env::vars().collect()
  }
  
  fn var(&self, key: &str) -> Result<String, String> {
    match env::var(&key) {
      Ok(val) => Ok(val),
      Err(_) => Err(format!("Variable {} does not exist", &key))
    }
  }
}

impl EnvConfig {
  fn new() -> Self {
    Self {}
  }
}

fn run(config: &EnvConfig) -> Result<bool, String>{
  let targets = config.get_all_targets()?;
  let timeout = config.get_timeout();

  for target in targets {
    let _ = target.poll(&timeout)?;
  }
  println!("All targets are up!");
  Ok(true)
}

fn main() {
    let config = EnvConfig::new();
    std::process::exit(match run(&config) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}
