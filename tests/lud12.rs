#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlp");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = lnurlkit::Server::new(addr.to_string())
        .pay_request(
            move |_| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::pay::Query {
                        callback,
                        short_description: String::new(),
                        long_description: None,
                        jpeg: None,
                        png: None,
                        comment_size: Some(140),
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        metadata_raw: None,
                    })
                }
            },
            |req: lnurlkit::pay::CallbackRequest| async move {
                Ok(lnurlkit::pay::CallbackResponse {
                    pr: format!("pierre:{}", req.comment),
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

    assert_eq!(pr.core.comment_size.unwrap(), 140);

    let invoice = pr
        .clone()
        .callback(314, String::new())
        .await
        .expect("callback");

    assert_eq!(invoice.pr, "pierre:");

    let invoice = pr
        .callback(314, String::from("comentario"))
        .await
        .expect("callback");

    assert_eq!(invoice.pr, "pierre:comentario");
}
