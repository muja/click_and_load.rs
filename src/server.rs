use iron::prelude::*;
use iron::status;
use router::Router;
use hyper::mime::{Mime, TopLevel, SubLevel};
use loader::Loader;

const CROSS_DOMAIN: &'static str = "<?xml version=\"1.0\"?>
<!DOCTYPE cross-domain-policy SYSTEM \"http://www.macromedia.com/xml/dtds/cross-domain-policy.dtd\">
<cross-domain-policy>
<allow-access-from domain=\"*\" />
</cross-domain-policy>
";

pub struct Server {
    router: Router
}

impl Server {
    pub fn new() -> Self {
        Server{
            router: Router::new()
        }
    }

    pub fn with_cnl(mut self) -> Self {
        self.router.get("/flash", |_: &mut Request| {
            Ok(Response::with((status::Ok, "UGET")))
        });
        self.router.get("/jdcheck.js", |_: &mut Request| {
            Ok(Response::with((status::Ok, "jdownloader = true;")).set(
                Mime(TopLevel::Text, SubLevel::Javascript, vec![])
            ))
        });
        self.router.get("/crossdomain.xml", |_: &mut Request| {
            Ok(Response::with((status::Ok, CROSS_DOMAIN)).set(
                Mime(TopLevel::Text, SubLevel::Html, vec![])
            ))
        });

        self.router.post("/flash/addcrypted2", Loader::click_and_load);
        self
    }

    pub fn run(self) {
        Iron::new(self.router).http("localhost:9666").unwrap();
    }
}
