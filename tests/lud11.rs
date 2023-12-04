#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlp");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = lnurlkit::server::Server::default()
        .pay_request(
            move || {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::core::pay_request::PayRequest {
                        callback,
                        short_description: String::new(),
                        long_description: None,
                        success_action: None,
                        jpeg: None,
                        png: None,
                        comment_size: 0,
                        min: 314,
                        max: 315,
                    })
                }
            },
            move |(amount, _)| async move {
                Ok(lnurlkit::core::pay_request::CallbackResponse {
                    pr: String::new(),
                    disposable: amount % 2 == 0,
                })
            },
        )
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::client::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr.clone().callback("", 314).await.expect("callback");
    assert!(invoice.disposable);

    let invoice = pr.callback("", 315).await.expect("callback");
    assert!(!invoice.disposable);
}
