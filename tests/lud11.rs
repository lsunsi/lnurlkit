#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlp");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = lnurlkit::Server::default()
        .pay_request(
            move |_| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::pay::server::Entrypoint {
                        callback,
                        short_description: String::new(),
                        long_description: None,
                        jpeg: None,
                        png: None,
                        comment_size: None,
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        currencies: None,
                        payer: None,
                    })
                }
            },
            |req: lnurlkit::pay::server::Callback| async move {
                Ok(lnurlkit::pay::server::CallbackResponse {
                    pr: String::new(),
                    disposable: matches!(req.amount, lnurlkit::pay::Amount::Millisatoshis(a) if a % 2 == 0),
                    success_action: None,
                })
            },
        )
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.entrypoint(&lnurl).await.expect("query");
    let lnurlkit::client::Entrypoint::Pay(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr
        .invoice(&lnurlkit::pay::Amount::Millisatoshis(314), None, None, None)
        .await
        .expect("callback");

    assert!(invoice.disposable);

    let invoice = pr
        .invoice(&lnurlkit::pay::Amount::Millisatoshis(315), None, None, None)
        .await
        .expect("callback");
    assert!(!invoice.disposable);
}
