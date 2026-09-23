//! Per-IP rate limit on every mutation: login, register, catalog edits,
//! ratings, uploads. Same budget as the original's `tower_governor`
//! preset — an 8-request burst refilling one every 500 ms.
//!
//! A middleware over the whole router rather than a layer per route,
//! because mutations are now server functions under `/api/…`; their
//! method is what tells them apart from reads (every read is a `GET`).

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroU32,
    sync::LazyLock,
    time::Duration,
};

use axum::{
    extract::{ConnectInfo, Request},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

static LIMITER: LazyLock<DefaultKeyedRateLimiter<IpAddr>> = LazyLock::new(|| {
    let quota = Quota::with_period(Duration::from_millis(500))
        .expect("non-zero period")
        .allow_burst(NonZeroU32::new(8).expect("non-zero burst"));
    RateLimiter::keyed(quota)
});

pub async fn limit_mutations(request: Request, next: Next) -> Response {
    let is_mutation = !matches!(*request.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if is_mutation {
        // Under `dx serve` there is no peer address (see
        // `server::serve_release`), so every dev request shares one bucket.
        let ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        if LIMITER.check_key(&ip).is_err() {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
        // Old keys would otherwise accumulate forever.
        if LIMITER.len() > 10_000 {
            LIMITER.retain_recent();
        }
    }
    next.run(request).await
}
