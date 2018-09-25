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

mod config;

pub fn hello() {
    println!("Hello world");
}

pub fn print_config() {
    let config = config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e));
    println!("Parsed config: {:?}", config);
}
