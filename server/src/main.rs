mod api;

use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use clap::Parser;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing::info;

const CLIENT_INDEX_HTML: &[u8] = include_bytes!("../../dist/index.html");
const CLIENT_JS: &[u8] = include_bytes!("../../dist/frontend.js");
const CLIENT_WASM: &[u8] = include_bytes!("../../dist/frontend_bg.wasm");

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "info")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// open the web-browser automatically on startup
    #[clap(long = "open")]
    open: bool,

    /// enable live-reload
    #[cfg_attr(debug_assertions, clap(long = "live-reload"))]
    live_reload: bool,
}

async fn client_index_html() -> Html<&'static [u8]> {
    Html(CLIENT_INDEX_HTML)
}

async fn client_js() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        CLIENT_JS,
    )
}

async fn client_wasm() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/wasm")],
        CLIENT_WASM,
    )
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    info!("listening on http://{sock_addr}");
    let mut app = api::router()
        .route("/hello", get(client_index_html))
        .route("/", get(client_index_html))
        .route("/frontend.js", get(client_js))
        .route("/frontend_bg.wasm", get(client_wasm));

    if opt.live_reload {
        info!("Live-Reload is enabled.");
        app = app.layer(LiveReloadLayer::new());
    }

    let listener = tokio::net::TcpListener::bind(&sock_addr)
        .await
        .expect(&format!("Error: unable to bind socket: {sock_addr}"));
    if opt.open {
        open::that(format!("http://{sock_addr}")).expect("Couldn't open web browser.");
    }
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .expect("Error: unable to start server");
}
