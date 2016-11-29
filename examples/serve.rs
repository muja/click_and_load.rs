extern crate click_and_load as cnl;
extern crate iron;
extern crate router;
extern crate env_logger;
#[macro_use]
extern crate log;
// extern crate hyper;

extern crate time;

use cnl::server;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Header;
use iron::headers::{LastModified, HttpDate};
use router::Router;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

fn print(_: &mut Router, subscribers: &mut Vec<Box<Fn(&Vec<String>)>>) {
    subscribers.push(Box::new(|links: &Vec<String>| {
        println!("{}", links.join("\n"));
    }));
}

fn last(router: &mut Router, subscribers: &mut Vec<Box<Fn(&Vec<String>)>>) {
    let last_links: Arc<Mutex<Option<(Vec<String>, time::Tm)>>> = Arc::new(Mutex::new(None));
    let s = last_links.clone();
    subscribers.push(Box::new(move |links: &Vec<String>| {
        let guard = s.lock();
        if let Ok(mut handle) = guard {
            *handle = Some((links.to_owned(), time::now()));
        }
    }));
    let r = last_links.clone();
    router.get("/last",
               move |_: &mut Request| {
        let guard = r.lock();
        if let Ok(handle) = guard {
            if let Some(ref links) = *handle {
                Ok(Response::with((status::Ok,
                                   Header(LastModified(HttpDate(links.1))),
                                   links.0.join("\n"))))
            } else {
                Ok(Response::with((status::NotFound, "")))
            }
        } else {
            Ok(Response::with((status::InternalServerError, "Failed to acquire lock")))
        }
    },
               "last");
}

fn next(router: &mut Router, cnl_subscribers: &mut Vec<Box<Fn(&Vec<String>)>>) {
    let subs: Arc<Mutex<Vec<Sender<Vec<String>>>>> = Arc::new(Mutex::new(Vec::new()));
    let x = subs.clone();
    cnl_subscribers.push(Box::new(move |links: &Vec<String>| {
        let guard = x.lock();
        if let Ok(mut handle) = guard {
            for sub in handle.to_owned() {
                sub.send(links.to_owned()).unwrap();
            }
            *handle = Vec::new();
        }
    }));
    let y = subs.clone();
    router.get("/next",
               move |_: &mut Request| {
        let recv = {
            let guard = y.lock();
            if let Ok(mut handle) = guard {
                let (send, recv) = mpsc::channel();
                handle.push(send);
                recv
            } else {
                return Ok(Response::with((status::InternalServerError, "Failed to acquire lock")));
            }
        };
        if let Ok(vec) = recv.recv() {
            Ok(Response::with((status::Ok, vec.join("\n"))))
        } else {
            warn!("COULDN'T RECEIVE");
            Ok(Response::with((status::InternalServerError, "Failed to acquire lock")))
        }
    },
               "next");
}

fn main() {
    env_logger::init().unwrap();
    let mut router = Router::new();
    let mut subscribers = Vec::new();
    last(&mut router, &mut subscribers);
    // print(&mut router, &mut subscribers);
    next(&mut router, &mut subscribers);
    for links in server::run_with(router).unwrap() {
        for sub in &subscribers {
            sub(&links);
        }
    }
}
