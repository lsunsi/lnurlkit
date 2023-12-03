#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = lnurlkit::client::Client::default();

    let queried = client
        .query("lnurl1dp68gurn8ghj7cnfwpsjuctswqhjuam9d3kz66mwdamkutmvde6hymrs9a4k2mn4cdry4t")
        .await
        .expect("query");

    println!("{queried:?}");

    let lnurlkit::client::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr.callback("comment", 123000).await.expect("callback");

    println!("{invoice:?}");
}
