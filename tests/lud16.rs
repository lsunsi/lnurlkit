#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = lnurlkit::Server::default()
        .pay_request(
            move |identifier: Option<String>| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::pay::server::Query {
                        callback,
                        short_description: String::from("today i become death"),
                        long_description: Some(String::from("the destroyer of worlds")),
                        jpeg: None,
                        png: None,
                        comment_size: None,
                        min: 314,
                        max: 315,
                        identifier: identifier.clone().filter(|i| i.starts_with('n')),
                        email: identifier.filter(|i| i.starts_with('j')),
                    })
                }
            },
            |req: lnurlkit::pay::server::CallbackRequest| async move {
                Ok(lnurlkit::pay::server::CallbackResponse {
                    pr: format!("pierre:{}", req.millisatoshis),
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

    let lnaddr = format!("nico@{addr}");
    let mut lnurl = lnurlkit::resolve(&lnaddr).expect("resolve");
    lnurl.set_scheme("http").expect("scheme");

    let bech32 = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&lnurl.as_ref()),
        bech32::Variant::Bech32,
    )
    .expect("bech32");

    let queried = client.query(&bech32).await.expect("query");
    let lnurlkit::client::Query::Pay(pr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(&pr.core.identifier.unwrap() as &str, "nico");

    let lnaddr = format!("jorel@{addr}");
    let mut lnurl = lnurlkit::resolve(&lnaddr).expect("resolve");
    lnurl.set_scheme("http").expect("scheme");

    let bech32 = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&lnurl.as_ref()),
        bech32::Variant::Bech32,
    )
    .expect("bech32");

    let queried = client.query(&bech32).await.expect("query");
    let lnurlkit::client::Query::Pay(pr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(&pr.core.email.unwrap() as &str, "jorel");
}
