extern crate click_and_load as cnl;
extern crate env_logger;

use cnl::server;

fn main() {
    env_logger::init().unwrap();
    for links in server::run().unwrap() {
        for link in links {
            println!("{}", link);
        }
    }
}
