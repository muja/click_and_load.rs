extern crate click_and_load as cnl;
extern crate env_logger;

use cnl::server;

fn main() {
    env_logger::init().unwrap();
    for link in server::run().unwrap() {
      println!("{}", link);
    }
}
