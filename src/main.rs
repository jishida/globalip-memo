#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_json;
#[cfg(not(test))]
extern crate serde_json;
extern crate reqwest;

mod logging;
#[macro_use]
mod error;
mod env;
mod util;
mod proc;

fn main() {
    logging::init_logger();

    match run() {
        Ok(..) => info!("globalip-memo: successfully completed"),
        Err(ref e) => {
            error!("globalip-memo: error - {}", e);
            let mut source = std::error::Error::source(e);
            loop {
                match source {
                    Some(err) => {
                        error!("globalip-memo: error source - {}", err);
                        source = err.source();
                    }
                    None => break,
                }
            }
        }
    }
}

fn run() -> error::Result<()> {
    info!("globalip-memo: start processing");
    let env = env::Env::new()?;
    let list = proc::fetch(&env);
    let addr = proc::process_fetch_result(list)?;
    let old_addr = proc::find_old_addr(&env);
    proc::output(&env, &addr, &old_addr.as_ref())?;
    Ok(())
}
