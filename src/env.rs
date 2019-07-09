use std::env::{current_dir, var};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::fmt;

use serde::Deserialize;

use super::error::Result;

const HOME_ENV: &str = "GLOBALIP_MEMO_HOME";
const CONFIG_FILENAME: &str = "globalip-config.json";
const OUTPUT_FILENAME: &str = "globalip.txt";

#[derive(Debug)]
enum WorkDir {
    Env,
    Current,
}

impl WorkDir {
    fn find(&self) -> Option<PathBuf> {
        match self {
            WorkDir::Env => var(HOME_ENV)
                .ok()
                .and_then(|path| PathBuf::from(path).canonicalize().ok()),
            WorkDir::Current => current_dir().ok(),
        }
    }
}

static WORK_DIRS: [WorkDir; 2] = [WorkDir::Env, WorkDir::Current];

fn resolve_dir() -> Result<PathBuf> {
    for work_dir in &WORK_DIRS {
        let dir = work_dir.find();
        if dir.is_some() {
            let d: PathBuf = dir.unwrap();
            debug!("resolve_dir: {:?} - {}", &work_dir, d.display());
            return Ok(d);
        } else {
            debug!("resolve_dir: {:?} - not found", &work_dir);
        }
    }
    Err(err!("Directory not found"))
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum IpVersion {
    #[serde(rename(deserialize = "ipv4"))]
    IPv4,
    #[serde(rename(deserialize = "ipv6"))]
    IPv6,
}

impl fmt::Display for IpVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpVersion::IPv4 => write!(f, "Ipv4"),
            IpVersion::IPv6 => write!(f, "Ipv6"),
        }
    }
}

impl Default for IpVersion {
    fn default() -> Self {
        IpVersion::IPv4
    }
}

fn default_weight() -> f64 {
    1f64
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Method {
    #[serde(rename(deserialize = "plain"))]
    Plain {
        url: String,
        #[serde(default)]
        regex: String,
        #[serde(default = "default_weight")]
        weight: f64,
    },
    #[serde(rename(deserialize = "json"))]
    Json {
        url: String,
        #[serde(default)]
        regex: String,
        path: String,
        #[serde(default = "default_weight")]
        weight: f64,
    },
}

impl Method {
    pub fn url(&self) -> &str {
        match self {
            Method::Plain { url, .. } => url,
            Method::Json { url, .. } => url,
        }
    }

    pub fn regex(&self) -> &str {
        match self {
            Method::Plain { regex, .. } => regex,
            Method::Json { regex, .. } => regex,
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Method::Plain { weight, .. } => *weight,
            Method::Json { weight, .. } => *weight,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    ip_version: IpVersion,
    methods: Vec<Method>,
}

impl Config {
    pub fn ip_version(&self) -> &IpVersion {
        &self.ip_version
    }

    pub fn methods(&self) -> &Vec<Method> {
        &self.methods
    }

    pub fn dns_strategy(&self) -> reqwest::LookupIpStrategy {
        match self.ip_version {
            IpVersion::IPv4 => reqwest::LookupIpStrategy::Ipv4Only,
            IpVersion::IPv6 => reqwest::LookupIpStrategy::Ipv6Only,
        }
    }
}

fn config_path(dir: &Path) -> Result<PathBuf> {
    let tmp = dir.join(CONFIG_FILENAME);
    let path = tmp.canonicalize().map_err(|e| {
        err_io!(e, "Failed to canonicalize config file path: {}", tmp.display())
    })?;
    if path.is_file() {
        Ok(path)
    } else {
        Err(err!("Config file not found: {}", path.display()))
    }
}

fn read_config<P: AsRef<Path>>(config_file: P) -> Result<Config> {
    let path = config_file.as_ref();
    let file = File::open(path).map_err(|e| {
        err_io!(e, "Faild to open config file: {}", path.display())
    })?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(|e| {
        err_json!(e, "Failed to parse config file: {}", path.display())
    }).and_then(|config: Config| {
        if config.methods().is_empty() {
            Err(err!("read_config: methods not found - {}", path.display()))
        } else {
            Ok(config)
        }
    })
}

#[derive(Debug)]
pub struct Env {
    dir: PathBuf,
    output_path: PathBuf,
    config: Config,
}

impl Env {
    pub fn new() -> Result<Self> {
        let dir = resolve_dir()?;
        let config_file = config_path(dir.as_path())?;
        let config = read_config(&config_file)?;
        let output_path = dir.join(OUTPUT_FILENAME);

        let env = Env {
            dir: dir,
            output_path: output_path,
            config: config,
        };
        debug!("Env::new: Environment loaded");
        debug!("{:?}", env);
        Ok(env)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn output_path(&self) -> &Path {
        self.output_path.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env::set_var;
    use std::path::PathBuf;
    #[test]
    fn test_env() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push(".test");
        set_var(HOME_ENV, dir);

        let res = Env::new();
        assert!(res.is_ok());
        let env = res.unwrap();
        println!("env: {:?}", env);
    }
}
