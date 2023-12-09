mod pay_request {
    #[test]
    fn alby() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/alby-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(pr.short_description, "Sats for lorenzo");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(255));
        assert_eq!(pr.max, 500_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://getalby.com/lnurlp/lorenzo/callback?amount=314"
        );
    }

    #[test]
    fn blink() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/blink-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(pr.short_description, "Payment to lorenzo");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(2000));
        assert_eq!(pr.max, 100_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://pay.mainnet.galoy.io/lnurlp/lorenzo/callback?amount=314"
        );
    }

    #[test]
    fn bipa() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/bipa-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(pr.short_description, "$kenu ⚡ bipa.app");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(140));
        assert_eq!(pr.max, 1_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert_eq!(pr.png.as_ref().unwrap().len(), 54697);

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://api.bipa.app/ln/request/invoice/kenu/1701784379/50n3BjOSWb1ZrxE9WmRcqlk2ylDzUJ1Q_GHN0pk_Q7Q/P6IMTO82jj6W21mUvXNgIlGmqGibx8MiaWfSjQ2wI88?amount=314"
        );
    }

    #[test]
    fn pouch() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/pouch-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(pr.short_description, "Lightning payment to ethan@pouch.ph");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(150));
        assert_eq!(pr.max, 10_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://app.pouch.ph/api/v2/lnurl/pay/ethan?amount=314"
        );
    }

    #[test]
    fn walletofsatoshi() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/walletofsatoshi-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.short_description,
            "Pay to Wallet of Satoshi user: wailingcity51"
        );
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(255));
        assert_eq!(pr.max, 100_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://livingroomofsatoshi.com/api/v1/lnurl/payreq/0e7f30e3-e74d-410d-bf86-50d101715e81?amount=314"
        );
    }

    #[test]
    fn zebedee() {
        let Ok(lnurlkit::Query::Pay(pr)) =
            include_str!("../fixtures/zebedee-pay.json").parse::<lnurlkit::Query>()
        else {
            panic!("parse");
        };

        assert_eq!(pr.short_description, "luhack - Welcome to my zbd.gg page!");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(150));
        assert_eq!(pr.max, 500_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert_eq!(pr.png.as_ref().unwrap().len(), 3993);

        assert_eq!(
            pr.callback(314, "").to_string(),
            "https://api.zebedee.io/v0/process-static-charges/8d648ac7-09f6-400c-8479-d05ac4d9d61d?amount=314"
        );
    }
}
