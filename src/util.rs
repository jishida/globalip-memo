use super::env::IpVersion;
use super::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn flatten(value: &Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    _flatten(value, "", &mut map);
    map
}

fn _flatten(value: &Value, path: &str, map: &mut HashMap<String, Value>) {
    match value {
        Value::Object(ref m) => {
            for (k, v) in m {
                let p = if path.is_empty() {
                    k.to_owned()
                } else {
                    format!("{}.{}", path, k)
                };
                _flatten(v, p.as_str(), map);
            }
        }
        Value::Array(ref arr) => {
            for (i, v) in arr.iter().enumerate() {
                let p = format!("{}[{}]", path, i);
                _flatten(v, p.as_str(), map);
            }
        }
        _ => {
            map.insert(path.to_owned(), value.clone());
        }
    }
}

pub fn parse_ip(ip_version: &IpVersion, s: &str) -> Result<IpAddr> {
    let trimmed = s.trim();
    match ip_version {
        IpVersion::IPv4 => trimmed.parse::<Ipv4Addr>().map(|addr| IpAddr::V4(addr)),
        IpVersion::IPv6 => trimmed.parse::<Ipv6Addr>().map(|addr| IpAddr::V6(addr)),
    }
    .map_err(|e| {
        err_addr!(e, "parse_ip: Failed to parse {} - {}", ip_version, s)
    })
}

pub fn get_body(url: &str, dns_strategy: reqwest::LookupIpStrategy) -> Result<String> {
    let client: reqwest::Client = reqwest::ClientBuilder::new()
        .dns_strategy(dns_strategy)
        .build()
        .map_err(|e| err_http!(e, "get_body: Failed to build http client - {}", url))?;
    client.get(url)
        .send()
        .and_then(|mut response| response.text())
        .map_err(|e| {
            err_http!(e, "get_body: Failed to get globalip - {}", url)
        })
}

pub fn parse_json(s: &str, path: &str) -> Result<String> {
    let value = serde_json::from_str::<serde_json::Value>(s)
        .map_err(|e| err_json!(e, "parse_json: Failed to parse JSON = {}", s))?;
    let map = flatten(&value);
    map.get(path)
        .and_then(|v| match v {
            serde_json::Value::String(ref t) => Some(t.to_owned()),
            _ => None,
        })
        .ok_or_else(|| err!(r#"parse_json: "{}" not found - {}"#, path, s))
}

pub fn extract_ip(s: &str, re: &str) -> Result<String> {
    if re.is_empty() {
        return Ok(s.to_owned());
    }
    regex::Regex::new(re)
        .map_err(|e| {
            err_regex!(e, "extract_ip: Invalid regulare expression = {}", re)
        })
        .and_then(|r: regex::Regex| {
            r.captures(s)
                .and_then(|c: regex::Captures| c.name("ip").map(|m| m.as_str().to_owned()))
                .ok_or_else(|| {
                    err!(r#"extract_ip: Failed to capture "ip" - regex: "{}", text: "{}""#, re, s)
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_flatten() {
        let text = r#"{
        "hoge": {
            "foo": {
                "arr": [
                    "0",
                    "1"
                ],
                "obj": {
                    "key": "value"
                },
                "num": 0,
                "boolean": false,
                "null": null
            }
        }
    }
    "#;
        let result = serde_json::from_str(text);
        assert!(result.is_ok());
        let value = result.unwrap();
        let map = flatten(&value);
        assert_eq!(map.len(), 6);
        assert_value(map.get("hoge.foo.arr[0]"), json!("0"));
        assert_value(map.get("hoge.foo.arr[1]"), json!("1"));
        assert_value(map.get("hoge.foo.obj.key"), json!("value"));
        assert_value(map.get("hoge.foo.num"), json!(0));
        assert_value(map.get("hoge.foo.boolean"), json!(false));
        assert_value(map.get("hoge.foo.null"), json!(null));
    }

    fn assert_value(actual: Option<&Value>, expected: Value) {
        assert!(actual.is_some());
        assert_eq!(actual.unwrap(), &expected);
    }
}
