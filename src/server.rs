use axum::{http::StatusCode, routing::get, Router};
use std::future::Future;

pub struct Server<P> {
    pub pay_request: Option<P>,
}

impl<P, PFut> Server<P>
where
    P: 'static + Send + Clone + Fn() -> PFut,
    PFut: Send + Future<Output = Result<crate::core::pay_request::PayRequest, StatusCode>>,
{
    pub fn build(self) -> Router<()> {
        let mut router = Router::new();

        if let Some(p) = self.pay_request {
            router = router.route(
                "/lnurlp",
                get(move || {
                    let p = p.clone();
                    async move { p().await.map(|a| a.to_string()) }
                }),
            );
        }

        router
    }
}
