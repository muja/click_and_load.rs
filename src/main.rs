extern crate iron;
extern crate crypto;
extern crate router;
extern crate hyper;
extern crate urlencoded;
extern crate rustc_serialize;

mod loader;
mod dukt;
mod server;
use server::Server;

fn main() {
    Server::new().with_cnl().run();
}
