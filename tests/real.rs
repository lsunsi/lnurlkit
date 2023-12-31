mod pay_request {
    #[test]
    fn alby() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/alby-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://getalby.com/lnurlp/lorenzo/callback"
        );
        assert_eq!(pr.short_description, "Sats for lorenzo");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(255));
        assert_eq!(pr.max, 500_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());
    }

    #[test]
    fn blink() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/blink-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://pay.mainnet.galoy.io/lnurlp/lorenzo/callback"
        );
        assert_eq!(pr.short_description, "Payment to lorenzo");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(2000));
        assert_eq!(pr.max, 100_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());
    }

    #[test]
    fn bipa() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/bipa-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://api.bipa.app/ln/request/invoice/kenu/1701784379/50n3BjOSWb1ZrxE9WmRcqlk2ylDzUJ1Q_GHN0pk_Q7Q/P6IMTO82jj6W21mUvXNgIlGmqGibx8MiaWfSjQ2wI88"
        );
        assert_eq!(pr.short_description, "$kenu ⚡ bipa.app");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(140));
        assert_eq!(pr.max, 1_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert_eq!(pr.png.as_ref().unwrap().len(), 54697);
    }

    #[test]
    fn pouch() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/pouch-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://app.pouch.ph/api/v2/lnurl/pay/ethan"
        );
        assert_eq!(pr.short_description, "Lightning payment to ethan@pouch.ph");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(150));
        assert_eq!(pr.max, 10_000_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert!(pr.png.is_none());
    }

    #[test]
    fn walletofsatoshi() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/walletofsatoshi-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://livingroomofsatoshi.com/api/v1/lnurl/payreq/0e7f30e3-e74d-410d-bf86-50d101715e81"
        );
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
    }

    #[test]
    fn zebedee() {
        let Ok(lnurlkit::Entrypoint::Pay(pr)) =
            (include_bytes!("../fixtures/zebedee-pay.json") as &[u8]).try_into()
        else {
            panic!("parse");
        };

        assert_eq!(
            pr.callback.as_str(),
            "https://api.zebedee.io/v0/process-static-charges/8d648ac7-09f6-400c-8479-d05ac4d9d61d"
        );
        assert_eq!(pr.short_description, "luhack - Welcome to my zbd.gg page!");
        assert!(pr.long_description.is_none());

        assert_eq!(pr.comment_size, Some(150));
        assert_eq!(pr.max, 500_000_000);
        assert_eq!(pr.min, 1000);

        assert!(pr.jpeg.is_none());
        assert_eq!(pr.png.as_ref().unwrap().len(), 3993);
    }
}
