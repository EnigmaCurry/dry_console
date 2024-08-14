mod api;
mod app_state;
mod response;
mod routing;

use crate::api::auth::Backend;
use app_state::SharedState;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, MethodRouter};
use axum::Router;
use clap::Parser;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing::info;
const API_PREFIX: &str = "/api";

pub type AppRouter = Router<SharedState>;
pub type AppMethodRouter = MethodRouter<SharedState, Infallible>;

////////////////////////////////////////////////////////////////////////////////
// static assets
////////////////////////////////////////////////////////////////////////////////
include!(concat!(env!("OUT_DIR"), "/generated_includes.rs"));
const CLIENT_INDEX_HTML: &[u8] = include_bytes!("../../dist/index.html");
const CLIENT_JS: &[u8] = include_bytes!("../../dist/frontend.js");
const CLIENT_WASM: &[u8] = include_bytes!("../../dist/frontend_bg.wasm");
const _SYSTEMD_UNIT: &[u8] = include_bytes!("../../systemd.service");

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

async fn serve_inline_file(
    content: &'static [u8],
    content_type: &'static str,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        content,
    )
}

////////////////////////////////////////////////////////////////////////////////
// Command line interface
////////////////////////////////////////////////////////////////////////////////
#[derive(Parser, Debug, Clone)]
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
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            format!(
                "{},hyper=info,mio=info,wasm_bindgen_wasm_interpreter=info",
                opt.log_level
            ),
        )
    }
    // enable console logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                // Suppress DEBUG logging for HTTP requests:
                .add_directive("tower_http::trace=info".parse().unwrap())
                .add_directive("axum::rejection=trace".parse().unwrap()),
        )
        .init();

    // Shutdown signal handler
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            // Notify all WebSocket handlers to shut down
            info!("Sending shutdown signal ...");
            let _ = shutdown_tx_clone.send(());
        }
    });

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    info!("listening on http://{sock_addr}");
    let shared_state = app_state::create_shared_state(&opt);
    let auth_backend = Backend::new(&shared_state);
    let inline_files = get_inline_files();
    let mut router = Router::new()
        .layer(routing::SlashRedirectLayer)
        .nest(API_PREFIX, api::router(auth_backend, shutdown_tx))
        .route("/", get(client_index_html))
        .route("/frontend.js", get(client_js))
        .route("/frontend_bg.wasm", get(client_wasm));
    for (name, content, content_type) in inline_files {
        router = router.route(name, get(move || serve_inline_file(content, content_type)));
    }
    let mut router = router
        .route("/*else", get(client_index_html))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state.clone());

    if cfg!(debug_assertions) {
        info!("Live-Reload is enabled.");
        router = router.layer(LiveReloadLayer::new());
    }

    // Finally, make the app into a service:
    let app = router.clone().into_make_service();

    //tracing::debug!("{:#?}", app);
    let listener = tokio::net::TcpListener::bind(&sock_addr)
        .await
        .unwrap_or_else(|_| panic!("Error: unable to bind socket: {sock_addr}"));
    let token = shared_state
        .clone()
        .read()
        .expect("Unable to read cache")
        .cache_get_string("token", "xxx");
    if opt.open {
        open::that(format!("http://{sock_addr}/login#token:{token}"))
            .expect("Couldn't open web browser.");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await.ok();
        })
        .await
        .expect("Error: unable to start server");
}
