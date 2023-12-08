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
                    Ok(lnurlkit::pay::Query {
                        callback,
                        short_description: String::from("today i become death"),
                        long_description: Some(String::from("the destroyer of worlds")),
                        jpeg: None,
                        png: None,
                        comment_size: None,
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        metadata_raw: None,
                    })
                }
            },
            |(amount, _)| async move {
                Ok(lnurlkit::pay::CallbackResponse {
                    pr: format!("pierre:{amount}"),
                    disposable: false,
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

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Query::Pay(pr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(pr.core.min, 314);
    assert_eq!(pr.core.max, 315);
    assert_eq!(pr.core.short_description, "today i become death");
    assert_eq!(
        pr.core.long_description.as_ref().unwrap(),
        "the destroyer of worlds"
    );

    let invoice = pr
        .callback(314, String::from("comment"))
        .await
        .expect("callback");

    assert_eq!(invoice.pr, "pierre:314");
}
