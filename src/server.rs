use axum::{
    extract::{Path, RawQuery},
    http::StatusCode,
    routing::get,
    Router,
};
use std::future::Future;

pub struct Server<AR, CE, CC, PE, PC, WE, WC> {
    auth_request: AR,
    channel_entrypoint: CE,
    channel_callback: CC,
    pay_entrypoint: PE,
    pay_callback: PC,
    withdraw_entrypoint: WE,
    withdraw_callback: WC,
}

impl Default
    for Server<
        // Auth
        unimplemented::Handler<crate::auth::server::Callback, crate::CallbackResponse>,
        // Channel Request
        unimplemented::Handler<(), crate::channel::server::Entrypoint>,
        unimplemented::Handler<crate::channel::server::Callback, crate::CallbackResponse>,
        // Pay Request
        unimplemented::Handler<Option<String>, crate::pay::server::Entrypoint>,
        unimplemented::Handler<crate::pay::server::Callback, crate::pay::server::CallbackResponse>,
        // Withdraw Request
        unimplemented::Handler<(), crate::withdraw::server::Entrypoint>,
        unimplemented::Handler<crate::withdraw::server::Callback, crate::CallbackResponse>,
    >
{
    fn default() -> Self {
        Server {
            auth_request: unimplemented::handler,

            channel_entrypoint: unimplemented::handler,
            channel_callback: unimplemented::handler,

            pay_entrypoint: unimplemented::handler,
            pay_callback: unimplemented::handler,

            withdraw_entrypoint: unimplemented::handler,
            withdraw_callback: unimplemented::handler,
        }
    }
}

impl<AR, CE, CC, PE, PC, WE, WC> Server<AR, CE, CC, PE, PC, WE, WC> {
    pub fn auth<AR2>(self, auth_request: AR2) -> Server<AR2, CE, CC, PE, PC, WE, WC> {
        Server {
            auth_request,
            channel_entrypoint: self.channel_entrypoint,
            channel_callback: self.channel_callback,
            pay_entrypoint: self.pay_entrypoint,
            pay_callback: self.pay_callback,
            withdraw_entrypoint: self.withdraw_entrypoint,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn channel_request<CE2, CC2>(
        self,
        channel_entrypoint: CE2,
        channel_callback: CC2,
    ) -> Server<AR, CE2, CC2, PE, PC, WE, WC> {
        Server {
            auth_request: self.auth_request,
            channel_entrypoint,
            channel_callback,
            pay_entrypoint: self.pay_entrypoint,
            pay_callback: self.pay_callback,
            withdraw_entrypoint: self.withdraw_entrypoint,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn pay_request<PE2, PC2>(
        self,
        pay_entrypoint: PE2,
        pay_callback: PC2,
    ) -> Server<AR, CE, CC, PE2, PC2, WE, WC> {
        Server {
            auth_request: self.auth_request,
            channel_entrypoint: self.channel_entrypoint,
            channel_callback: self.channel_callback,
            pay_entrypoint,
            pay_callback,
            withdraw_entrypoint: self.withdraw_entrypoint,
            withdraw_callback: self.withdraw_callback,
        }
    }

    pub fn withdraw_request<WE2, WC2>(
        self,
        withdraw_entrypoint: WE2,
        withdraw_callback: WC2,
    ) -> Server<AR, CE, CC, PE, PC, WE2, WC2> {
        Server {
            auth_request: self.auth_request,
            channel_entrypoint: self.channel_entrypoint,
            channel_callback: self.channel_callback,
            pay_entrypoint: self.pay_entrypoint,
            pay_callback: self.pay_callback,
            withdraw_entrypoint,
            withdraw_callback,
        }
    }
}

impl<AR, ARFut, CE, CQFut, CC, CCFut, PE, PEFut, PC, PCFut, WE, WEFut, WC, WCFut>
    Server<AR, CE, CC, PE, PC, WE, WC>
where
    AR: 'static + Send + Clone + Fn(crate::auth::server::Callback) -> ARFut,
    ARFut: Send + Future<Output = Result<crate::CallbackResponse, StatusCode>>,

    CE: 'static + Send + Clone + Fn(()) -> CQFut,
    CQFut: Send + Future<Output = Result<crate::channel::server::Entrypoint, StatusCode>>,

    CC: 'static + Send + Clone + Fn(crate::channel::server::Callback) -> CCFut,
    CCFut: Send + Future<Output = Result<crate::CallbackResponse, StatusCode>>,

    PE: 'static + Send + Clone + Fn(Option<String>) -> PEFut,
    PEFut: Send + Future<Output = Result<crate::pay::server::Entrypoint, StatusCode>>,

    PC: 'static + Send + Clone + Fn(crate::pay::server::Callback) -> PCFut,
    PCFut: Send + Future<Output = Result<crate::pay::server::CallbackResponse, StatusCode>>,

    WE: 'static + Send + Clone + Fn(()) -> WEFut,
    WEFut: Send + Future<Output = Result<crate::withdraw::server::Entrypoint, StatusCode>>,

    WC: 'static + Send + Clone + Fn(crate::withdraw::server::Callback) -> WCFut,
    WCFut: Send + Future<Output = Result<crate::CallbackResponse, StatusCode>>,
{
    #[allow(clippy::too_many_lines)]
    pub fn build(self) -> Router<()> {
        Router::new()
            .route(
                "/keyauth",
                get(move |RawQuery(q): RawQuery| {
                    let ar = self.auth_request.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        ar(p).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
                    }
                }),
            )
            .route(
                "/lnurlc",
                get(move || {
                    let ce = self.channel_entrypoint.clone();
                    async move {
                        ce(()).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
                    }
                }),
            )
            .route(
                "/lnurlc/callback",
                get(move |RawQuery(q): RawQuery| {
                    let cc = self.channel_callback.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        cc(p).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
                    }
                }),
            )
            .route(
                "/.well-known/lnurlp/:identifier",
                get({
                    let pe = self.pay_entrypoint.clone();
                    move |Path(identifier): Path<String>| {
                        let pe = pe.clone();
                        async move {
                            pe(Some(identifier)).await.and_then(|a| {
                                Vec::<u8>::try_from(a)
                                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                            })
                        }
                    }
                }),
            )
            .route(
                "/lnurlp",
                get(move || {
                    let pe = self.pay_entrypoint.clone();
                    async move {
                        pe(None).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
                    }
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
                    let we = self.withdraw_entrypoint.clone();
                    async move {
                        we(()).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
                    }
                }),
            )
            .route(
                "/lnurlw/callback",
                get(move |RawQuery(q): RawQuery| {
                    let wc = self.withdraw_callback.clone();
                    async move {
                        let q = q.ok_or(StatusCode::BAD_REQUEST)?;
                        let p = q.as_str().try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
                        wc(p).await.and_then(|a| {
                            Vec::<u8>::try_from(a).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                        })
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
