use std::println;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = lnurl_kit::Lnurl::default();

    let queried = client
        .query("lnurl1dp68gurn8ghj7cnfwpsjuctswqhjuam9d3kz66mwdamkutmvde6hymrs9a4k2mn4cdry4t")
        .await
        .expect("query");

    println!("{queried:?}");

    let lnurl_kit::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr.callback(123000).await.expect("callback");

    println!("{invoice}");
}
