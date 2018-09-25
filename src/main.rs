extern crate env_logger;
extern crate payments_lib;

fn main() {
    env_logger::init();
    payments_lib::print_config();
}
