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

pub fn mount(router: &mut Router) -> Receiver<String> {
    router.get("/flash",
               |_: &mut Request| Ok(Response::with((status::Ok, "UGET"))));
    router.get("/jdcheck.js", |_: &mut Request| {
        Ok(Response::with((status::Ok, "jdownloader = true;"))
               .set(Mime(TopLevel::Text, SubLevel::Javascript, vec![])))
    });
    router.get("/crossdomain.xml", |_: &mut Request| {
        Ok(Response::with((status::Ok, CROSS_DOMAIN))
               .set(Mime(TopLevel::Text, SubLevel::Html, vec![])))
    });

    let (send, recv) = mpsc::channel();
    let loader = Loader { sender: Mutex::new(send) };

    router.post("/flash/addcrypted2", loader);

    recv
}

pub fn run() -> Result<Receiver<String>, HttpError> {
    let mut router = Router::new();
    let rec = mount(&mut router);
    let iron = try!(Iron::new(router).http("0.0.0.0:9666"));
    thread::spawn(move || {
        let stderr = ::std::io::stderr();
        writeln!(stderr.lock(), "Listening on 0.0.0.0:9666!").unwrap();
        iron
    });
    Ok(rec)
}
