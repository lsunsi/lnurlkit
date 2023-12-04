use crate::core;
use axum::{extract::RawQuery, http::StatusCode, routing::get, Router};
use std::future::Future;

pub struct Server<CQ, CC, WQ, WC, PQ, PC> {
    channel_query: CQ,
    channel_callback: CC,
    withdraw_query: WQ,
    withdraw_callback: WC,
    pay_query: PQ,
    pay_callback: PC,
}

impl Default
    for Server<
        unimplemented::Handler0<core::channel_request::ChannelRequest>,
        unimplemented::Handler1<(String, String), core::channel_request::CallbackResponse>,
        unimplemented::Handler0<core::withdraw_request::WithdrawRequest>,
        unimplemented::Handler1<String, core::withdraw_request::CallbackResponse>,
        unimplemented::Handler0<core::pay_request::PayRequest>,
        unimplemented::Handler1<(u64, Option<String>), core::pay_request::CallbackResponse>,
    >
{
    fn default() -> Self {
        Server {
            channel_query: unimplemented::handler0,
            channel_callback: unimplemented::handler1,
            withdraw_query: unimplemented::handler0,
            withdraw_callback: unimplemented::handler1,
            pay_query: unimplemented::handler0,
            pay_callback: unimplemented::handler1,
        }
    }
}

impl<CQ, CC, WQ, WC, PQ, PC> Server<CQ, CC, WQ, WC, PQ, PC> {
    pub fn channel_request<CQ2, CC2>(
        self,
        channel_query: CQ2,
        channel_callback: CC2,
    ) -> Server<CQ2, CC2, WQ, WC, PQ, PC> {
        Server {
            channel_query,
            channel_callback,
            pay_query: self.pay_query,
            pay_callback: self.pay_callback,
            withdraw_query: self.withdraw_query,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn withdraw_request<WQ2, WC2>(
        self,
        withdraw_query: WQ2,
        withdraw_callback: WC2,
    ) -> Server<CQ, CC, WQ2, WC2, PQ, PC> {
        Server {
            channel_query: self.channel_query,
            channel_callback: self.channel_callback,
            pay_query: self.pay_query,
            pay_callback: self.pay_callback,
            withdraw_query,
            withdraw_callback,
        }
    }

    pub fn pay_request<PQ2, PC2>(
        self,
        pay_query: PQ2,
        pay_callback: PC2,
    ) -> Server<CQ, CC, WQ, WC, PQ2, PC2> {
        Server {
            channel_query: self.channel_query,
            channel_callback: self.channel_callback,
            pay_query,
            pay_callback,
            withdraw_query: self.withdraw_query,
            withdraw_callback: self.withdraw_callback,
        }
    }
}

impl<CQ, CQFut, CC, CCFut, WQ, WQFut, WC, WCFut, PQ, PQFut, PC, PCFut>
    Server<CQ, CC, WQ, WC, PQ, PC>
where
    CQ: 'static + Send + Clone + Fn() -> CQFut,
    CQFut: Send + Future<Output = Result<core::channel_request::ChannelRequest, StatusCode>>,

    CC: 'static + Send + Clone + Fn((String, String)) -> CCFut,
    CCFut: Send + Future<Output = Result<core::channel_request::CallbackResponse, StatusCode>>,

    WQ: 'static + Send + Clone + Fn() -> WQFut,
    WQFut: Send + Future<Output = Result<core::withdraw_request::WithdrawRequest, StatusCode>>,

    WC: 'static + Send + Clone + Fn(String) -> WCFut,
    WCFut: Send + Future<Output = Result<core::withdraw_request::CallbackResponse, StatusCode>>,

    PQ: 'static + Send + Clone + Fn() -> PQFut,
    PQFut: Send + Future<Output = Result<core::pay_request::PayRequest, StatusCode>>,

    PC: 'static + Send + Clone + Fn((u64, Option<String>)) -> PCFut,
    PCFut: Send + Future<Output = Result<core::pay_request::CallbackResponse, StatusCode>>,
{
    pub fn build(self) -> Router<()> {
        Router::new()
            .route(
                "/lnurlc",
                get(move || {
                    let cq = self.channel_query.clone();
                    async move { cq().await.map(|a| a.to_string()) }
                }),
            )
            .route(
                "/lnurlc/callback",
                get(move |RawQuery(q): RawQuery| {
                    let cc = self.channel_callback.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let k1 = qs.get("k1").ok_or(StatusCode::BAD_REQUEST)?;
                        let remoteid = qs.get("remoteid").ok_or(StatusCode::BAD_REQUEST)?;
                        cc((String::from(*k1), String::from(*remoteid)))
                            .await
                            .map(|a| a.to_string())
                    }
                }),
            )
            .route(
                "/lnurlw",
                get(move || {
                    let wq = self.withdraw_query.clone();
                    async move { wq().await.map(|a| a.to_string()) }
                }),
            )
            .route(
                "/lnurlw/callback",
                get(move |RawQuery(q): RawQuery| {
                    let wc = self.withdraw_callback.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let pr = qs.get("pr").ok_or(StatusCode::BAD_REQUEST)?;
                        wc(String::from(*pr)).await.map(|a| a.to_string())
                    }
                }),
            )
            .route(
                "/lnurlp",
                get(move || {
                    let pq = self.pay_query.clone();
                    async move { pq().await.map(|a| a.to_string()) }
                }),
            )
            .route(
                "/lnurlp/callback",
                get(move |RawQuery(q): RawQuery| {
                    let pc = self.pay_callback.clone();
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

#[cfg(test)]
mod tests {
    #[test]
    fn default_builds() {
        drop(super::Server::default().build());
    }
}
