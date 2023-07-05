use axum::routing::IntoMakeService;
use axum::Router;
use futures::channel::mpsc::Receiver;
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use std::sync::Arc;
use tokio::sync::Mutex;

const CROSS_DOMAIN: &'static str = r#"<?xml version="1.0"?>
<!DOCTYPE cross-domain-policy SYSTEM "http://www.macromedia.com/xml/dtds/cross-domain-policy.dtd">
<cross-domain-policy>
<allow-access-from domain="*"/>
</cross-domain-policy>"#;

pub fn mount(router: axum::Router) -> (Receiver<Vec<String>>, axum::Router) {
    let (send, recv) = futures::channel::mpsc::channel(1);
    let loader =
        axum::handler::Handler::with_state(crate::loader::handle, Arc::new(Mutex::new(send)));
    (
        recv,
        router
            .route("/flash", axum::routing::get(|| async { "CNL.RS" }))
            .route(
                "/jdcheck.js",
                axum::routing::get(|| async { "jdownloader = true;" }),
            )
            .route(
                "/crossdomain.xml",
                axum::routing::get(|| async { CROSS_DOMAIN }),
            )
            .route_service("/flash/addcrypted2", loader),
    )
}

pub fn server(addr: Option<&str>) -> hyper::server::Builder<AddrIncoming> {
    hyper::Server::bind(&addr.unwrap_or("0.0.0.0:9666").parse().unwrap())
}

pub fn run() -> (
    Receiver<Vec<String>>,
    Server<AddrIncoming, IntoMakeService<Router>>,
) {
    let router = axum::Router::new();
    let (rec, router) = mount(router);
    let server = server(None).serve(router.into_make_service());
    (rec, server)
}