use click_and_load::server as CnlServer;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            "trace,hyper=info"
                .parse::<tracing_subscriber::EnvFilter>()
                .unwrap(),
        )
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let (mut recv, server) = CnlServer::run();
    let handle = tokio::spawn(server);
    while let Some(links) = recv.next().await {
        for link in links {
            println!("{link}");
        }
    }
    handle.await.unwrap().unwrap();
}
