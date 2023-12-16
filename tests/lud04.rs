#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let url = format!("http://{addr}/keyauth?tag=login&k1=3031323334353637383930313233343536373839303132333435363738393031");

    let router = lnurlkit::Server::default()
        .auth(move |req: lnurlkit::auth::server::Callback| async move {
            if &req.sig == b"0123456789012345678901234567890101234567890123456789012345678901" {
                Ok(lnurlkit::CallbackResponse::Ok)
            } else {
                Ok(lnurlkit::CallbackResponse::Error {
                    reason: String::from("bad sig"),
                })
            }
        })
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&url),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.entrypoint(&lnurl).await.expect("query");
    let lnurlkit::client::Entrypoint::Auth(a) = queried else {
        panic!("not pay request");
    };

    assert_eq!(a.core.url.as_str(), url);
    assert_eq!(&a.core.k1, b"01234567890123456789012345678901");
    assert!(a.core.action.is_none());

    let response = a
        .auth(
            "pierre",
            b"0123456789012345678901234567890101234567890123456789012345678901",
        )
        .await
        .expect("callback");

    assert!(matches!(response, lnurlkit::CallbackResponse::Ok));
}
