use std::thread;
use iron::prelude::*;
use iron::status;
use iron::error::HttpError;
use router::Router;
use hyper::mime::{Mime, TopLevel, SubLevel};
use loader::Loader;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

const CROSS_DOMAIN: &'static str = "<?xml version=\"1.0\"?>
<!DOCTYPE cross-domain-policy SYSTEM \
                                    \"http://www.macromedia.com/xml/dtds/cross-domain-policy.\
                                    dtd\">
<cross-domain-policy>
<allow-access-from domain=\"*\" \
                                    />
</cross-domain-policy>
";

pub fn mount(router: &mut Router) -> Receiver<Vec<String>> {
    let (send, recv) = mpsc::channel();
    let loader = Loader { sender: Mutex::new(send) };
    router.get("/flash",
               |_: &mut Request| Ok(Response::with((status::Ok, "CNL.RS"))),
               "flash");
    router.get("/jdcheck.js",
               |_: &mut Request| {
                   Ok(Response::with((status::Ok, "jdownloader = true;"))
                       .set(Mime(TopLevel::Text, SubLevel::Javascript, vec![])))
               },
               "jdcheck");
    router.get("/crossdomain.xml",
               |_: &mut Request| {
                   Ok(Response::with((status::Ok, CROSS_DOMAIN))
                       .set(Mime(TopLevel::Text, SubLevel::Html, vec![])))
               },
               "crossdomain");
    router.post("/flash/addcrypted2", loader, "addcrypted2");

    recv
}

pub fn run_with(mut router: Router) -> Result<Receiver<Vec<String>>, HttpError> {
    let rec = mount(&mut router);
    let iron = try!(Iron::new(router).http("0.0.0.0:9666"));
    thread::spawn(move || {
        let stderr = ::std::io::stderr();
        writeln!(stderr.lock(), "Listening on 0.0.0.0:9666!").unwrap();
        iron
    });
    Ok(rec)
}

pub fn run() -> Result<Receiver<Vec<String>>, HttpError> {
    let router = Router::new();
    run_with(router)
}
