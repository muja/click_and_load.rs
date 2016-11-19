extern crate click_and_load as cnl;
extern crate env_logger;
extern crate iron;
extern crate router;
// extern crate hyper;

use cnl::server;
use iron::prelude::*;
use iron::status;
use router::Router;
use std::sync::Mutex;
use std::sync::Arc;

fn main() {
    env_logger::init().unwrap();
    let last = Arc::new(Mutex::new(None::<Vec<String>>));
    let last1 = last.clone();
    let mut router = Router::new();
    router.get("/last", move |_: &mut Request| {
        let links = last1.lock();
        if let Ok(guard) = links {
            if let Some(ref links) = *guard {
                Ok(Response::with((status::Ok, links.join("\n"))))
            } else {
                Ok(Response::with((status::NotFound, "")))
            }
        } else {
            Ok(Response::with((status::InternalServerError, "")))
        }
    });

    for links in server::run_with(router).unwrap() {
        for link in &links {
            println!("{}", link);
        }
        *last.lock().unwrap() = Some(links);
    }
}
