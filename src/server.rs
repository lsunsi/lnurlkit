use axum::{
    extract::{Path, RawQuery},
    http::StatusCode,
    routing::get,
    Router,
};
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
        unimplemented::Handler<(), crate::channel::server::Response>,
        unimplemented::Handler<
            crate::channel::server::CallbackQuery,
            crate::channel::server::CallbackResponse,
        >,
        // Pay Request
        unimplemented::Handler<Option<String>, crate::pay::server::Response>,
        unimplemented::Handler<
            crate::pay::server::CallbackQuery,
            crate::pay::server::CallbackResponse,
        >,
        // Withdraw Request
        unimplemented::Handler<(), crate::withdraw::server::Response>,
        unimplemented::Handler<
            crate::withdraw::server::CallbackQuery,
            crate::withdraw::server::CallbackResponse,
        >,
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
    CQFut: Send + Future<Output = Result<crate::channel::server::Response, StatusCode>>,

    CC: 'static + Send + Clone + Fn(crate::channel::server::CallbackQuery) -> CCFut,
    CCFut: Send + Future<Output = Result<crate::channel::server::CallbackResponse, StatusCode>>,

    PQ: 'static + Send + Clone + Fn(Option<String>) -> PQFut,
    PQFut: Send + Future<Output = Result<crate::pay::server::Response, StatusCode>>,

    PC: 'static + Send + Clone + Fn(crate::pay::server::CallbackQuery) -> PCFut,
    PCFut: Send + Future<Output = Result<crate::pay::server::CallbackResponse, StatusCode>>,

    WQ: 'static + Send + Clone + Fn(()) -> WQFut,
    WQFut: Send + Future<Output = Result<crate::withdraw::server::Response, StatusCode>>,

    WC: 'static + Send + Clone + Fn(crate::withdraw::server::CallbackQuery) -> WCFut,
    WCFut: Send + Future<Output = Result<crate::withdraw::server::CallbackResponse, StatusCode>>,
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
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        cc(p).await.map(|a| a.to_string())
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
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        pc(p).await.map(|a| a.to_string())
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
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        wc(p).await.map(|a| a.to_string())
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
