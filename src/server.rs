use axum::{
    extract::{Path, RawQuery},
    http::StatusCode,
    http::Uri,
    routing::get,
    Router,
};
use std::future::Future;

pub struct Server<CQ, CC, PQ, PC, WQ, WC> {
    base: String,
    channel_query: CQ,
    channel_callback: CC,
    pay_query: PQ,
    pay_callback: PC,
    withdraw_query: WQ,
    withdraw_callback: WC,
}

impl
    Server<
        // Channel Request
        unimplemented::Handler<(), crate::channel::Query>,
        unimplemented::Handler<crate::channel::CallbackRequest, crate::channel::CallbackResponse>,
        // Pay Request
        unimplemented::Handler<Option<String>, crate::pay::Query>,
        unimplemented::Handler<crate::pay::CallbackRequest, crate::pay::CallbackResponse>,
        // Withdraw Request
        unimplemented::Handler<(), crate::withdraw::Query>,
        unimplemented::Handler<crate::withdraw::CallbackRequest, crate::withdraw::CallbackResponse>,
    >
{
    #[must_use]
    pub fn new(base: String) -> Self {
        Server {
            base,

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
            base: self.base,
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
            base: self.base,
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
            base: self.base,
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
    CQFut: Send + Future<Output = Result<crate::channel::Query, StatusCode>>,

    CC: 'static + Send + Clone + Fn(crate::channel::CallbackRequest) -> CCFut,
    CCFut: Send + Future<Output = Result<crate::channel::CallbackResponse, StatusCode>>,

    PQ: 'static + Send + Clone + Fn(Option<String>) -> PQFut,
    PQFut: Send + Future<Output = Result<crate::pay::Query, StatusCode>>,

    PC: 'static + Send + Clone + Fn(crate::pay::CallbackRequest) -> PCFut,
    PCFut: Send + Future<Output = Result<crate::pay::CallbackResponse, StatusCode>>,

    WQ: 'static + Send + Clone + Fn(()) -> WQFut,
    WQFut: Send + Future<Output = Result<crate::withdraw::Query, StatusCode>>,

    WC: 'static + Send + Clone + Fn(crate::withdraw::CallbackRequest) -> WCFut,
    WCFut: Send + Future<Output = Result<crate::withdraw::CallbackResponse, StatusCode>>,
{
    #[allow(clippy::too_many_lines)]
    pub fn build(self) -> Router<()> {
        let base_c = self.base.clone();
        let base_p = self.base.clone();
        let base_w = self.base.clone();

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
                get(move |uri: Uri, RawQuery(q): RawQuery| {
                    let cc = self.channel_callback.clone();
                    async move {
                        let url = url::Url::parse(&format!("https://{base_c}{}", uri.path()))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let k1 = String::from(*qs.get("k1").ok_or(StatusCode::BAD_REQUEST)?);
                        let remoteid =
                            String::from(*qs.get("remoteid").ok_or(StatusCode::BAD_REQUEST)?);

                        let req = if qs.get("cancel").copied() == Some("1") {
                            Some(crate::channel::CallbackRequest::Cancel { url, remoteid, k1 })
                        } else {
                            match qs.get("private").copied() {
                                Some("0") => Some(crate::channel::CallbackRequest::Accept {
                                    url,
                                    remoteid,
                                    k1,
                                    private: false,
                                }),
                                Some("1") => Some(crate::channel::CallbackRequest::Accept {
                                    url,
                                    remoteid,
                                    k1,
                                    private: true,
                                }),
                                _ => None,
                            }
                        }
                        .ok_or(StatusCode::BAD_REQUEST)?;

                        cc(req).await.map(|a| a.to_string())
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
                get(move |uri: Uri, RawQuery(q): RawQuery| {
                    let pc = self.pay_callback.clone();
                    async move {
                        let url = url::Url::parse(&format!("https://{base_p}{}", uri.path()))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let millisatoshis = qs
                            .get("amount")
                            .and_then(|s| s.parse().ok())
                            .ok_or(StatusCode::BAD_REQUEST)?;

                        let comment = qs
                            .get("comment")
                            .map(|c| String::from(*c))
                            .unwrap_or_default();

                        pc(crate::pay::CallbackRequest {
                            url,
                            comment,
                            millisatoshis,
                        })
                        .await
                        .map(|a| a.to_string())
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
                get(move |uri: Uri, RawQuery(q): RawQuery| {
                    let wc = self.withdraw_callback.clone();
                    async move {
                        let url = url::Url::parse(&format!("https://{base_w}{}", uri.path()))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let qs = q
                            .split('&')
                            .filter_map(|s| s.split_once('='))
                            .collect::<std::collections::BTreeMap<_, _>>();

                        let k1 = String::from(*qs.get("k1").ok_or(StatusCode::BAD_REQUEST)?);
                        let pr = String::from(*qs.get("pr").ok_or(StatusCode::BAD_REQUEST)?);

                        wc(crate::withdraw::CallbackRequest { url, k1, pr })
                            .await
                            .map(|a| a.to_string())
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
        drop(super::Server::new(String::from("base:31415")).build());
    }
}
