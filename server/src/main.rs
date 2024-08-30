mod api;
mod app_state;
mod response;
mod routing;
mod sudo;

use crate::api::auth::Backend;
use api::workstation::platform::detect_toolbox;
use app_state::SharedState;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, MethodRouter};
use axum::Router;
use clap::ArgAction;
use clap::Parser;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::process;
use std::process::exit;
use std::str::FromStr;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing::{debug, error, info, warn};
use uzers::get_current_uid;

const API_PREFIX: &str = "/api";

pub type AppRouter = Router<SharedState>;
pub type AppMethodRouter = MethodRouter<SharedState, Infallible>;

////////////////////////////////////////////////////////////////////////////////
// static assets
////////////////////////////////////////////////////////////////////////////////
include!(concat!(env!("OUT_DIR"), "/generated_includes.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_command_library.rs"));
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
#[clap(
    name = "server",
    about = "dry_console is your interactive workstation controller for Docker and d.rymcg.tech."
)]
pub struct Opt {
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

    /// Acquire root privileges via sudo and maintain its session indefinitely (This feature is activated automatically if the host is a toolbox container)
    #[clap(long = "sudo", action = ArgAction::SetTrue)]
    sudo: bool,

    /// Explicitly disable sudo (overrides --sudo)
    #[clap(long = "no-sudo", action = ArgAction::SetTrue)]
    no_sudo: bool,

    /// Timeout for sudo authentication, in seconds
    #[clap(long = "sudo-timeout-seconds", default_value = "60")]
    sudo_timeout_seconds: u64,

    /// Refresh interval to keep sudo session alive, in seconds
    #[clap(long = "sudo-refresh-interval", default_value = "60")]
    sudo_refresh_interval: u64,
}

impl Opt {
    fn resolve_sudo(&self) -> Option<bool> {
        if self.no_sudo {
            // Disable via --no-sudo explicitly
            Some(false)
        } else if self.sudo {
            // Enable sudo via --sudo
            Some(true)
        } else {
            // Sudo is unset by args, but may be enabled because of toolbox containership
            None
        }
    }
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

    // Make sure this is not run as root
    if get_current_uid() == 0 {
        error!("This program should not be run as root. Use the --sudo argument instead.");
        process::exit(1);
    }

    let shared_state = app_state::create_shared_state(&opt);

    // Acquire root privilege only if configured to do so, unless the
    // host is detected to be a toolbox or distrobox container, in
    // which case the feature should be enabled by default:
    match opt.resolve_sudo() {
        Some(true) => {
            warn!("Root access will now be requested via sudo:");
            match sudo::acquire_sudo(opt.sudo_timeout_seconds).await {
                Ok(_) => {
                    tokio::spawn(async move {
                        sudo::keep_sudo_session_alive(
                            opt.sudo_refresh_interval,
                            opt.sudo_timeout_seconds,
                        )
                        .await;
                    });
                }
                Err(e) => {
                    error!("Failed to acquire sudo authentication: {}", e);
                    exit(1);
                }
            };
            {
                let mut state = shared_state.write().await;
                state.sudo_enabled = true;
            }
        }
        Some(false) => {}
        None => {
            if detect_toolbox() {
                match sudo::acquire_sudo(2).await {
                    Ok(_) => {
                        warn!("A toolbox-like container was detected, therefore container level root access is acquired automatically via sudo.");
                    }
                    Err(e) => {
                        error!("A toolbox-like container was detected, but there was an unexpected failure to acquire sudo privileges :: {}", e);
                        exit(1);
                    }
                }
                {
                    let mut state = shared_state.write().await;
                    state.sudo_enabled = true;
                }
            }
        }
    }

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
    let auth_backend = Backend::new(&shared_state);
    let inline_files = get_inline_files();
    let mut router = Router::new()
        .layer(routing::SlashRedirectLayer)
        .nest(
            API_PREFIX,
            api::router(auth_backend, shutdown_tx, State(shared_state.clone())),
        )
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
    let token;
    {
        token = shared_state
            .clone()
            .read()
            .await
            .cache_get_string("token", "xxx");
    }
    if opt.open {
        open::that(format!("http://{sock_addr}/login#token:{token}"))
            .expect("Couldn't open web browser.");
    }

    debug!("now");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await.ok();
        })
        .await
        .expect("Error: unable to start server");
}
