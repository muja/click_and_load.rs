use click_and_load as cnl;
#[macro_use]
extern crate tracing;
// extern crate hyper;

use cnl::server;
use futures::channel::mpsc::{self, UnboundedSender};
use futures::{StreamExt, SinkExt};
use hyper::StatusCode;
use std::sync::Arc;
use std::sync::Mutex;

fn last(router: axum::Router, subscribers: &mut Vec<Box<dyn Fn(Vec<String>)>>) -> axum::Router {
    let last_links: Arc<Mutex<Option<(Vec<String>, time::OffsetDateTime)>>> =
        Arc::new(Mutex::new(None));
    let s = last_links.clone();
    subscribers.push(Box::new(move |links: Vec<String>| {
        let s = s.clone();
        let mut handle = s.lock().unwrap();
        *handle = Some((links.to_owned(), time::OffsetDateTime::now_utc()));
    }));
    let r = last_links.clone();
    router.route(
        "/last",
        axum::routing::get(|| async move {
            let handle = r.lock().unwrap();
            if let Some(ref links) = *handle {
                Ok(links.0.join("\n"))
            } else {
                Err((StatusCode::NOT_FOUND, "Not found"))
            }
        }),
    )
}

fn next(router: axum::Router, cnl_subscribers: &mut Vec<Box<dyn Fn(Vec<String>)>>) -> axum::Router {
    let subs: Arc<Mutex<Vec<UnboundedSender<Vec<String>>>>> = Arc::new(Mutex::new(Vec::new()));
    let sub = subs.clone();
    cnl_subscribers.push(Box::new(move |links: Vec<String>| {
        let mut handle = sub.lock().unwrap();
        for mut sub in handle.to_owned() {
            let links = links.to_owned();
            tokio::spawn(async move {
                sub.send(links).await.unwrap()
            });
        }
        *handle = Vec::new();
    }));
    let subs = subs.clone();
    router.route(
        "/next",
        axum::routing::get(|| async move {
            let mut recv = {
                let (send, recv) = mpsc::unbounded::<Vec<String>>();
                subs.lock().unwrap().push(send);
                recv
            };
            if let Some(vec) = recv.next().await {
                Ok(vec.join("\n"))
            } else {
                warn!("COULDN'T RECEIVE");
                Err("Failed to acquire lock")
            }
        }),
    )
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            "trace"
                .parse::<tracing_subscriber::EnvFilter>()
                .unwrap(),
        )
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let mut subscribers = Vec::new();
    let mut router = axum::Router::new();
    router = last(router, &mut subscribers);
    router = next(router, &mut subscribers);
    let (mut recv, router) = server::mount(router);
    let handle = tokio::spawn(server::server(None).serve(router.into_make_service()));
    while let Some(links) = recv.next().await {
        for sub in &subscribers {
            sub(links.clone());
        }
    }
    handle.await.unwrap().unwrap();
}
