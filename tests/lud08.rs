#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let callback = url::Url::parse(&format!("http://{addr}/lnurlw/callback")).expect("url");
    let callback2 = url::Url::parse(&format!("http://{addr}/lnurlw/callback")).expect("url");

    let w = lnurlkit::withdraw::server::Response {
        description: String::from("descricao"),
        k1: String::from("caum"),
        callback: callback.clone(),
        min: 314,
        max: 315,
    };

    let query_url_slow = format!("http://{addr}/lnurlw");
    let query_url_fast = format!("{query_url_slow}?{w}");

    let router = lnurlkit::Server::default()
        .withdraw_request(
            move |()| {
                let callback = callback.clone();
                async move {
                    Ok(lnurlkit::withdraw::server::Response {
                        description: String::from("outra-descricao"),
                        k1: String::from("cadois"),
                        callback,
                        min: 123,
                        max: 321,
                    })
                }
            },
            |_: lnurlkit::withdraw::server::CallbackQuery| async { unimplemented!() },
        )
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url_slow),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Response::Withdraw(wr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(wr.core.min, 123);
    assert_eq!(wr.core.max, 321);
    assert_eq!(&wr.core.description as &str, "outra-descricao");
    assert_eq!(wr.core.callback, callback2);

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url_fast),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Response::Withdraw(wr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(wr.core.min, 314);
    assert_eq!(wr.core.max, 315);
    assert_eq!(&wr.core.description as &str, "descricao");
    assert_eq!(wr.core.callback, callback2);
}
