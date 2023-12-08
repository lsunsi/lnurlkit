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
                        comment_size: None,
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
                    pr: String::new(),
                    disposable: false,
                    success_action: if req.millisatoshis == 0 {
                        None
                    } else if req.millisatoshis == 1 {
                        Some(lnurlkit::pay::SuccessAction::Message(req.comment))
                    } else {
                        Some(lnurlkit::pay::SuccessAction::Url(
                            url::Url::parse("http://u.rl").expect("url"),
                            req.comment,
                        ))
                    },
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

    let invoice = pr
        .clone()
        .callback(0, String::new())
        .await
        .expect("callback");

    assert!(invoice.success_action.is_none());

    let invoice = pr
        .clone()
        .callback(1, String::from("mensagem"))
        .await
        .expect("callback");

    let Some(lnurlkit::pay::SuccessAction::Message(m)) = invoice.success_action else {
        panic!("bad success action");
    };

    assert_eq!(m, "mensagem");

    let invoice = pr
        .callback(2, String::from("descricao"))
        .await
        .expect("callback");

    let Some(lnurlkit::pay::SuccessAction::Url(u, d)) = invoice.success_action else {
        panic!("bad success action");
    };

    assert_eq!(u.to_string(), "http://u.rl/");
    assert_eq!(d, "descricao");
}
