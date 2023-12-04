use axum::{extract::RawQuery, http::StatusCode, routing::get, Router};
use std::future::Future;

pub struct Server<PQ, PC> {
    pay_request_query: PQ,
    pay_request_callback: PC,
}

impl Default
    for Server<
        unimplemented::Handler0<crate::core::pay_request::PayRequest>,
        unimplemented::Handler1<(u64, Option<String>), crate::core::pay_request::CallbackResponse>,
    >
{
    fn default() -> Self {
        Server {
            pay_request_query: unimplemented::handler0,
            pay_request_callback: unimplemented::handler1,
        }
    }
}

impl<PQ, PC> Server<PQ, PC> {
    pub fn pay_request<PQ2, PC2>(
        self,
        pay_request_query: PQ2,
        pay_request_callback: PC2,
    ) -> Server<PQ2, PC2> {
        Server {
            pay_request_query,
            pay_request_callback,
        }
    }
}

impl<PQ, PQFut, PC, PCFut> Server<PQ, PC>
where
    PQ: 'static + Send + Clone + Fn() -> PQFut,
    PQFut: Send + Future<Output = Result<crate::core::pay_request::PayRequest, StatusCode>>,
    PC: 'static + Send + Clone + Fn((u64, Option<String>)) -> PCFut,
    PCFut: Send + Future<Output = Result<crate::core::pay_request::CallbackResponse, StatusCode>>,
{
    pub fn build(self) -> Router<()> {
        Router::new()
            .route(
                "/lnurlp",
                get(move || {
                    let pq = self.pay_request_query.clone();
                    async move { pq().await.map(|a| a.to_string()) }
                }),
            )
            .route(
                "/lnurlp/callback",
                get(move |RawQuery(q): RawQuery| {
                    let pc = self.pay_request_callback.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let amount = qs
                            .get("amount")
                            .and_then(|s| s.parse().ok())
                            .ok_or(StatusCode::BAD_REQUEST)?;

                        let comment = qs.get("comment").map(|c| String::from(*c));

                        pc((amount, comment)).await.map(|a| a.to_string())
                    }
                }),
            )
    }
}

mod unimplemented {
    use axum::http::StatusCode;
    use std::{
        future::Future,
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    pub(super) type Handler0<T> = fn() -> Unimplemented<T>;
    pub(super) type Handler1<T1, T> = fn(T1) -> Unimplemented<T>;

    pub struct Unimplemented<T>(PhantomData<T>);

    impl<T> Future for Unimplemented<T> {
        type Output = Result<T, StatusCode>;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<T, StatusCode>> {
            Poll::Ready(Err(StatusCode::NOT_IMPLEMENTED))
        }
    }

    pub(super) fn handler0<T>() -> Unimplemented<T> {
        Unimplemented(PhantomData)
    }

    pub(super) fn handler1<T, T1>(_: T1) -> Unimplemented<T> {
        Unimplemented(PhantomData)
    }
}
