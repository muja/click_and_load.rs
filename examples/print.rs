extern crate click_and_load as cnl;

use cnl::server::Server;

fn main() {
    Server::new().with_cnl().run();
}
