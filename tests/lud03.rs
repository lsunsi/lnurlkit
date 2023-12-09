#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlw");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlw/callback")).expect("url");

    let router = lnurlkit::Server::default()
        .withdraw_request(
            move |()| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::withdraw::server::Query {
                        description: String::from("descricao"),
                        k1: String::from("caum"),
                        callback,
                        min: 314,
                        max: 315,
                    })
                }
            },
            |req: lnurlkit::withdraw::server::CallbackRequest| async move {
                Ok(if &req.pr as &str == "pierre" {
                    lnurlkit::withdraw::server::CallbackResponse::Ok
                } else {
                    lnurlkit::withdraw::server::CallbackResponse::Error {
                        reason: req.k1.to_string(),
                    }
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
    let lnurlkit::client::Query::Withdraw(wr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(wr.core.min, 314);
    assert_eq!(wr.core.max, 315);
    assert_eq!(&wr.core.description as &str, "descricao");

    let response = wr.callback("pierre").await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::withdraw::client::CallbackResponse::Ok
    ));

    let response = wr.callback("pierrado").await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::withdraw::client::CallbackResponse::Error { reason } if &reason as &str == "caum"
    ));
}
