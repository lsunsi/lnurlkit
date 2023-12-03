#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlp");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = {
        let mut router = lnurlkit::server::builder();

        router.pay_request = Some(move || {
            let callback = callback_url.clone();

            async {
                Ok(lnurlkit::core::pay_request::PayRequest {
                    callback,
                    short_description: String::from("today i become death"),
                    long_description: Some(String::from("the destroyer of worlds")),
                    success_action: None,
                    jpeg: None,
                    png: None,
                    comment_size: 0,
                    min: 314,
                    max: 315,
                })
            }
        });

        router.build()
    };

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

    println!("{queried:?}");

    let lnurlkit::client::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(pr.core.min, 314);
    assert_eq!(pr.core.max, 315);
    assert_eq!(pr.core.short_description, "today i become death");
    assert_eq!(pr.core.long_description.unwrap(), "the destroyer of worlds");

    // let invoice = pr.callback("comment", 123000).await.expect("callback");

    // println!("{invoice:?}");
}
