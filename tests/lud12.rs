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
                        comment_size: Some(140),
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        currencies: None,
                    })
                }
            },
            |req: lnurlkit::pay::server::Callback| async move {
                Ok(lnurlkit::pay::server::CallbackResponse {
                    pr: format!("pierre:{:?}", req.comment),
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

    let queried = client.entrypoint(&lnurl).await.expect("query");
    let lnurlkit::client::Entrypoint::Pay(pr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(pr.core.comment_size.unwrap(), 140);

    let invoice = pr
        .invoice(&lnurlkit::pay::Amount::Millisatoshis(314), None, None)
        .await
        .expect("callback");

    assert_eq!(&invoice.pr as &str, "pierre:None");

    let invoice = pr
        .invoice(
            &lnurlkit::pay::Amount::Millisatoshis(314),
            Some("comentario"),
            None,
        )
        .await
        .expect("callback");

    assert_eq!(&invoice.pr as &str, "pierre:Some(\"comentario\")");
}
