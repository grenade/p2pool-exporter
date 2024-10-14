use axum::{routing::get, Extension, Router};
use clap::Parser;
use std::net::{
    IpAddr,
    SocketAddr,
};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use utils::{
    //serve_json_metrics,
    serve_prometheus_metrics,
    serve_stratum_table,
};

mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short='d',
        long,
        value_name = "data directory",
        default_value = "/var/lib/p2pool",
        help_heading = "p2pool",
        help = "the p2pool data directory"
    )]
    data_directory: PathBuf,

    #[arg(
        short='i',
        long,
        value_name = "ip address",
        default_value = "127.0.0.1",
        help_heading = "http server",
        help = "the ip address to listen on"
    )]
    listen_ip: IpAddr,

    #[arg(
        short='p',
        long,
        value_name = "port",
        default_value = "18090",
        help_heading = "http server",
        help = "the port to listen on"
    )]
    listen_port: u16,

    #[arg(
        short='m',
        long,
        value_name = "metrics path",
        default_value = "/metrics",
        help_heading = "http server",
        help = "the path portion of the url to prometheus metrics"
    )]
    metrics_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("reading from directory: {}", args.data_directory.display());

    let app = Router::new()
        .route("/", get(serve_stratum_table))
        .route(args.metrics_path.into_os_string().to_str().unwrap(), get(serve_prometheus_metrics))
        //.route("/json", get(serve_json_metrics))
        .layer(Extension(args.data_directory));

    let socket = SocketAddr::from((args.listen_ip, args.listen_port));
    info!("listening on {}", &socket);
    axum::Server::bind(&socket).serve(app.into_make_service()).await.unwrap();
}
