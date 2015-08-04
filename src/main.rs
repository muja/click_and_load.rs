extern crate iron;
extern crate crypto;
extern crate router;
extern crate hyper;
extern crate urlencoded;
extern crate rustc_serialize;
extern crate duktape_sys;

pub mod loader;
pub mod dukt;
pub mod server;
use server::Server;

fn main() {
    Server::new().with_cnl().run();
}
