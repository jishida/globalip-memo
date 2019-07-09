use crate::env::{Config, Env, Method};
use crate::error::Result;
use crate::util;
use std::cell::Cell;
use std::fs::File;
use std::io::{Read, Write};
use std::net::IpAddr;

const OUTPUT_MAX_SIZE: usize = 39;

fn get_globalip(method: &Method, config: &Config) -> Result<IpAddr> {
    let mut body = util::get_body(method.url(), config.dns_strategy())?;
    match method {
        Method::Json {path, ..} => body = util::parse_json(body.as_str(), path)?,
        _ => {}
    }
    let ip = util::extract_ip(body.as_str(), method.regex())?;
    util::parse_ip(config.ip_version(), ip.as_str())
}

pub fn fetch(env: &Env) -> Vec<(&Method, Result<IpAddr>)> {
    let mut list = Vec::with_capacity(env.config().methods().len());
    for method in env.config().methods() {
        let result = get_globalip(method, env.config());
        debug!("fetch: result - {:?},  method - {:?}", &result, method);
        list.push((method, result));
    }
    list
}

pub fn process_fetch_result(list: Vec<(&Method, Result<IpAddr>)>) -> Result<IpAddr> {
    let results: Vec<(&Method, &IpAddr)> = list.iter()
        .filter_map(|(&ref method, addr_result)| {
            match addr_result {
                Ok(addr) => Some((method, addr)),
                Err(e) => {
                    warn!("process_fetch_result: Failed to fetch method - {}", method.url());
                    warn!("process_fetch_result: error - {}", e);
                    let mut source = std::error::Error::source(e);
                    loop {
                        match source {
                            Some(err) => {
                                warn!("process_fetch_result: error source - {}", err);
                                source = err.source();
                            }
                            None => break,
                        }
                    }
                    None
                }
            }
        }).collect();
    if results.is_empty() {
        return Err(err!("process_fetch_result: Global IP address not found"));
    }
    let mut counter = Vec::<(&IpAddr, Cell<f64>)>::new();
    for (method, addr) in &results {
        info!("process_fetch_result: Global IP address {} found - {}", addr, method.url());
        match counter.iter().find(|t| &t.0 == addr) {
            Some((_, c)) => c.set(c.get() + method.weight()),
            None => counter.push((addr, Cell::new(method.weight()))),
        };
    }
    counter.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
    debug!("process_fetch_result: counter - {:?}", &counter);
    if counter.len() > 1 {
        warn!("process_fetch_result: Different addresses detected");
        for (&addr, count) in &counter {
            warn!("process_fetch_result: address - {}, count - {}", addr, count.get());
        }
    }
    Ok(counter[0].0.clone())
}

pub fn find_old_addr(env: &Env) -> Option<IpAddr> {
    let path = env.output_path();
    if !path.is_file() {
        debug!("find_old_addr: output file not found - {}", path.display());
        return None;
    }
    let mut file = File::open(path)
        .map_err(|e| {
                warn!(
                    "find_old_addr: Failed to open previous output - {}",
                    path.display()
                );
                warn!("find_old_addr: open error - {}", e);
        })
        .ok()?;
    let mut buf = [0u8; OUTPUT_MAX_SIZE + 1];
    let len = Read::read(&mut file, &mut buf)
        .map_err(|e| {
            warn!(
                "find_old_addr: Failed to read previous output - {}",
                path.display()
            );
            warn!("find_previous: read error - {}", e);
        })
        .ok()?;
    if len > OUTPUT_MAX_SIZE {
        warn!("find_old_addr: Previous output too large - {}", path.display());
        return None;
    }
    let ip = String::from_utf8(buf[..len].to_vec())
        .map_err(|e| {
            warn!("find_old_addr: Failed to decode previous output - {}", path.display());
            warn!("find_old_addr: decode error - {}", e);
        })
        .ok()?;
    let addr = util::parse_ip(env.config().ip_version(), ip.as_str())
        .map_err(|e| {
            warn!("find_old_addr: Failed to parse previous output - {}", ip.as_str());
            warn!("find_old_addr: parse error - {}", e);
        })
        .ok()?;
    debug!("find_old_addr: Previous IP address found - {}", &addr);
    Some(addr)
}

pub fn output(env: &Env, addr: &IpAddr, old_addr: &Option<&IpAddr>) -> Result<()> {
    if old_addr.is_some() && old_addr.unwrap() == addr {
        info!("output: Up to date - {}", addr);
        return Ok(());
    }
    let path = env.output_path();
    let mut file = File::create(path)
        .map_err(|e| err_io!(e, "output: Failed to create output file - {}", path.display()))?;
    let mut buf = format!("{}", addr).into_bytes();
    file.write_all(buf.as_mut())
        .map_err(|e| err_io!(e, "output: Failed to write {} to {}", addr, path.display()))?;
    match old_addr {
        Some(old) => info!("output: Updated {} to {} - {}", old, addr, path.display()),
        None => info!("output: Updated to {} - {}", addr, path.display()),
    }
    Ok(())
}
