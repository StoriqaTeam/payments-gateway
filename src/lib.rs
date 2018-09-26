extern crate futures;
#[macro_use]
extern crate failure;
extern crate hyper;
#[macro_use]
extern crate diesel;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate config as config_crate;
#[macro_use]
extern crate http_router;
extern crate hyper_tls;
extern crate regex;

#[macro_use]
mod macros;
mod api;
mod config;
mod types;
mod utils;

use config::Config;

pub fn hello() {
    println!("Hello world");
}

pub fn print_config() {
    println!("Parsed config: {:?}", get_config());
}

pub fn start_server() {
    let config = get_config();
    api::start_server(config);
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
