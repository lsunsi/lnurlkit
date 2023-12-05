use miniserde::{
    de::Visitor,
    make_place,
    ser::{Fragment, Serialize},
    Deserialize, Error, Result,
};

make_place!(Place);

#[derive(Debug, Clone)]
pub(crate) struct Url<'a>(pub(crate) std::borrow::Cow<'a, url::Url>);

impl Visitor for Place<Url<'_>> {
    fn string(&mut self, s: &str) -> Result<()> {
        let url = url::Url::parse(s).map_err(|_| Error)?;
        self.out = Some(Url(std::borrow::Cow::Owned(url)));
        Ok(())
    }
}

impl Deserialize for Url<'_> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

impl Serialize for Url<'_> {
    fn begin(&self) -> Fragment {
        Fragment::Str(self.0.to_string().into())
    }
}
