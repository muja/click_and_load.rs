extern crate click_and_load as cnl;
extern crate env_logger;

use cnl::server::Server;

fn main() {
    env_logger::init().unwrap();
    Server::new().with_cnl().run();
}
