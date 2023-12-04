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
                        jpeg: None,
                        png: None,
                        comment_size: 0,
                        min: 314,
                        max: 315,
                    })
                }
            },
            move |(amount, comment): (u64, Option<String>)| async move {
                Ok(lnurlkit::core::pay_request::CallbackResponse {
                    pr: String::new(),
                    disposable: false,
                    success_action: if amount == 0 {
                        None
                    } else if amount == 1 {
                        Some(lnurlkit::core::pay_request::SuccessAction::Message(
                            comment.unwrap_or_default(),
                        ))
                    } else {
                        Some(lnurlkit::core::pay_request::SuccessAction::Url(
                            url::Url::parse("http://u.rl").expect("url"),
                            comment.unwrap_or_default(),
                        ))
                    },
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

    let invoice = pr.clone().callback("", 0).await.expect("callback");
    assert!(invoice.success_action.is_none());

    let invoice = pr.clone().callback("mensagem", 1).await.expect("callback");

    let Some(lnurlkit::core::pay_request::SuccessAction::Message(m)) = invoice.success_action
    else {
        panic!("bad success action");
    };

    assert_eq!(m, "mensagem");

    let invoice = pr.callback("descricao", 2).await.expect("callback");

    let Some(lnurlkit::core::pay_request::SuccessAction::Url(u, d)) = invoice.success_action else {
        panic!("bad success action");
    };

    assert_eq!(u.to_string(), "http://u.rl/");
    assert_eq!(d, "descricao");
}