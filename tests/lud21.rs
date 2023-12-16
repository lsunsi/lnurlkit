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
                        short_description: String::from("today i become death"),
                        long_description: Some(String::from("the destroyer of worlds")),
                        jpeg: None,
                        png: None,
                        comment_size: None,
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        currencies: Some(vec![
                            lnurlkit::pay::Currency {
                                code: String::from("BRL"),
                                name: String::from("Reais"),
                                symbol: String::from("R$"),
                                decimals: 2,
                                multiplier: 314.15,
                                convertible: true,
                            },
                            lnurlkit::pay::Currency {
                                code: String::from("USD"),
                                name: String::from("Dólar"),
                                symbol: String::from("$"),
                                decimals: 3,
                                multiplier: 123.321,
                                convertible: false,
                            },
                        ]),
                    })
                }
            },
            |req: lnurlkit::pay::server::Callback| async move {
                Ok(lnurlkit::pay::server::CallbackResponse {
                    pr: format!("pierre:{:?}:{:?}", req.amount, req.convert),
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

    let currencies = pr.core.currencies.as_ref().unwrap();

    assert_eq!(currencies[0].code, "BRL");
    assert_eq!(currencies[0].name, "Reais");
    assert_eq!(currencies[0].symbol, "R$");
    assert_eq!(currencies[0].decimals, 2);
    assert!((currencies[0].multiplier - 314.15).abs() < f64::EPSILON);
    assert!(currencies[0].convertible);

    assert_eq!(currencies[1].code, "USD");
    assert_eq!(currencies[1].name, "Dólar");
    assert_eq!(currencies[1].symbol, "$");
    assert_eq!(currencies[1].decimals, 3);
    assert!((currencies[1].multiplier - 123.321).abs() < f64::EPSILON);
    assert!(!currencies[1].convertible);

    let invoice = pr
        .invoice(
            &lnurlkit::pay::Amount::Currency(String::from("USD"), 314),
            None,
            Some("BRL"),
        )
        .await
        .expect("callback");

    assert_eq!(
        &invoice.pr as &str,
        "pierre:Currency(\"USD\", 314):Some(\"BRL\")"
    );
}
