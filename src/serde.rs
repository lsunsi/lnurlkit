use miniserde::{de::Visitor, make_place, Deserialize, Error, Result};

make_place!(Place);

#[derive(Debug, Clone)]
pub(crate) struct Url(pub(crate) url::Url);

impl Visitor for Place<Url> {
    fn string(&mut self, s: &str) -> Result<()> {
        let url = url::Url::parse(s).map_err(|_| Error)?;
        self.out = Some(Url(url));
        Ok(())
    }
}

impl Deserialize for Url {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}
