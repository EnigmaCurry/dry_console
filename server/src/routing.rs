use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::app_state::SharedState;
use crate::response::AppError;

use axum::http::StatusCode;
use axum::routing::MethodRouter;
use axum::Router;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SlashRedirectLayer;

impl<S> Layer<S> for SlashRedirectLayer {
    type Service = SlashRedirect<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SlashRedirect { inner }
    }
}

#[derive(Clone)]
pub struct SlashRedirect<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for SlashRedirect<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let uri = req.uri().to_string();
        if !uri.ends_with('/') {
            let new_uri = format!("{}/", uri);
            let response = Response::builder()
                .status(StatusCode::PERMANENT_REDIRECT)
                .header("Location", new_uri)
                .body(Body::empty())
                .unwrap();

            Box::pin(async move { Ok(response) })
        } else {
            let fut = self.inner.call(req);
            Box::pin(fut)
        }
    }
}

pub fn route(
    path: &str,
    method_router: MethodRouter<SharedState, Infallible>,
) -> Router<SharedState> {
    let p: String = match path.trim_matches('/') {
        "" => "/".to_string(),
        p2 => format!("/{}/", p2),
    };
    //debug!("{:?}", p);
    Router::new().route(&p, method_router)
}
