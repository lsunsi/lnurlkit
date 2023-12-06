use axum::{extract::Path, extract::RawQuery, http::StatusCode, routing::get, Router};
use std::future::Future;

pub struct Server<CQ, CC, PQ, PC, WQ, WC> {
    channel_query: CQ,
    channel_callback: CC,
    pay_query: PQ,
    pay_callback: PC,
    withdraw_query: WQ,
    withdraw_callback: WC,
}

impl Default
    for Server<
        // Channel Request
        unimplemented::Handler<(), crate::core::channel::Query>,
        unimplemented::Handler<
            (String, String, crate::core::channel::CallbackAction),
            crate::core::channel::CallbackResponse,
        >,
        // Pay Request
        unimplemented::Handler<Option<String>, crate::core::pay::Query>,
        unimplemented::Handler<(u64, Option<String>), crate::core::pay::CallbackResponse>,
        // Withdraw Request
        unimplemented::Handler<(), crate::core::withdraw::Query>,
        unimplemented::Handler<(String, String), crate::core::withdraw::CallbackResponse>,
    >
{
    fn default() -> Self {
        Server {
            channel_query: unimplemented::handler,
            channel_callback: unimplemented::handler,

            pay_query: unimplemented::handler,
            pay_callback: unimplemented::handler,

            withdraw_query: unimplemented::handler,
            withdraw_callback: unimplemented::handler,
        }
    }
}

impl<CQ, CC, PQ, PC, WQ, WC> Server<CQ, CC, PQ, PC, WQ, WC> {
    pub fn channel_request<CQ2, CC2>(
        self,
        channel_query: CQ2,
        channel_callback: CC2,
    ) -> Server<CQ2, CC2, PQ, PC, WQ, WC> {
        Server {
            channel_query,
            channel_callback,
            pay_query: self.pay_query,
            pay_callback: self.pay_callback,
            withdraw_query: self.withdraw_query,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn pay_request<PQ2, PC2>(
        self,
        pay_query: PQ2,
        pay_callback: PC2,
    ) -> Server<CQ, CC, PQ2, PC2, WQ, WC> {
        Server {
            channel_query: self.channel_query,
            channel_callback: self.channel_callback,
            pay_query,
            pay_callback,
            withdraw_query: self.withdraw_query,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn withdraw_request<WQ2, WC2>(
        self,
        withdraw_query: WQ2,
        withdraw_callback: WC2,
    ) -> Server<CQ, CC, PQ, PC, WQ2, WC2> {
        Server {
            channel_query: self.channel_query,
            channel_callback: self.channel_callback,
            pay_query: self.pay_query,
            pay_callback: self.pay_callback,
            withdraw_query,
            withdraw_callback,
        }
    }
}

impl<CQ, CQFut, CC, CCFut, PQ, PQFut, PC, PCFut, WQ, WQFut, WC, WCFut>
    Server<CQ, CC, PQ, PC, WQ, WC>
where
    CQ: 'static + Send + Clone + Fn(()) -> CQFut,
    CQFut: Send + Future<Output = Result<crate::core::channel::Query, StatusCode>>,

    CC: 'static
        + Send
        + Clone
        + Fn((String, String, crate::core::channel::CallbackAction)) -> CCFut,
    CCFut: Send + Future<Output = Result<crate::core::channel::CallbackResponse, StatusCode>>,

    PQ: 'static + Send + Clone + Fn(Option<String>) -> PQFut,
    PQFut: Send + Future<Output = Result<crate::core::pay::Query, StatusCode>>,

    PC: 'static + Send + Clone + Fn((u64, Option<String>)) -> PCFut,
    PCFut: Send + Future<Output = Result<crate::core::pay::CallbackResponse, StatusCode>>,

    WQ: 'static + Send + Clone + Fn(()) -> WQFut,
    WQFut: Send + Future<Output = Result<crate::core::withdraw::Query, StatusCode>>,

    WC: 'static + Send + Clone + Fn((String, String)) -> WCFut,
    WCFut: Send + Future<Output = Result<crate::core::withdraw::CallbackResponse, StatusCode>>,
{
    #[allow(clippy::too_many_lines)]
    pub fn build(self) -> Router<()> {
        Router::new()
            .route(
                "/lnurlc",
                get(move || {
                    let cq = self.channel_query.clone();
                    async move { cq(()).await.map(|a| a.to_string()) }
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
                        let action = qs
                            .get("cancel")
                            .filter(|v| **v == "1")
                            .map(|_| crate::core::channel::CallbackAction::Cancel)
                            .or_else(|| {
                                qs.get("private").and_then(|v| match *v {
                                    "0" => Some(crate::core::channel::CallbackAction::Accept {
                                        private: false,
                                    }),
                                    "1" => Some(crate::core::channel::CallbackAction::Accept {
                                        private: true,
                                    }),
                                    _ => None,
                                })
                            })
                            .ok_or(StatusCode::BAD_REQUEST)?;

                        let param = (String::from(*k1), String::from(*remoteid), action);
                        cc(param).await.map(|a| a.to_string())
                    }
                }),
            )
            .route(
                "/.well-known/lnurlp/:identifier",
                get({
                    let pq = self.pay_query.clone();
                    move |Path(identifier): Path<String>| {
                        let pq = pq.clone();
                        async move { pq(Some(identifier)).await.map(|a| a.to_string()) }
                    }
                }),
            )
            .route(
                "/lnurlp",
                get(move || {
                    let pq = self.pay_query.clone();
                    async move { pq(None).await.map(|a| a.to_string()) }
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
            .route(
                "/lnurlw",
                get(move || {
                    let wq = self.withdraw_query.clone();
                    async move { wq(()).await.map(|a| a.to_string()) }
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

                        let k1 = qs.get("k1").ok_or(StatusCode::BAD_REQUEST)?;
                        let pr = qs.get("pr").ok_or(StatusCode::BAD_REQUEST)?;

                        let param = (String::from(*k1), String::from(*pr));
                        wc(param).await.map(|a| a.to_string())
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

    pub(super) type Handler<Param, Ret> = fn(Param) -> Unimplemented<Ret>;
    pub(super) fn handler<Param, Ret>(_: Param) -> Unimplemented<Ret> {
        Unimplemented(PhantomData)
    }

    pub struct Unimplemented<T>(PhantomData<T>);

    impl<T> Future for Unimplemented<T> {
        type Output = Result<T, StatusCode>;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<T, StatusCode>> {
            Poll::Ready(Err(StatusCode::NOT_IMPLEMENTED))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_builds() {
        drop(super::Server::default().build());
    }
}
