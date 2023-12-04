#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlw");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlw/callback")).expect("url");

    let router = lnurlkit::server::Server::default()
        .withdraw_request(
            move || {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::core::withdraw_request::WithdrawRequest {
                        description: String::from("descricao"),
                        k1: String::from("caum"),
                        callback,
                        min: 314,
                        max: 315,
                    })
                }
            },
            |pr| async move {
                Ok(if pr == "pierre" {
                    lnurlkit::core::withdraw_request::CallbackResponse::Ok
                } else {
                    lnurlkit::core::withdraw_request::CallbackResponse::Error(pr)
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
    let lnurlkit::client::Query::WithdrawRequest(wr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(wr.core.min, 314);
    assert_eq!(wr.core.max, 315);
    assert_eq!(wr.core.description, "descricao");

    let response = wr.clone().callback("pierre").await.expect("callback");
    assert!(matches!(
        response,
        lnurlkit::core::withdraw_request::CallbackResponse::Ok
    ));

    let response = wr.callback("pierrado").await.expect("callback");
    assert!(matches!(
        response,
        lnurlkit::core::withdraw_request::CallbackResponse::Error(r) if r == "pierrado"
    ));
}