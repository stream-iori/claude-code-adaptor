pub(crate) mod request_adapter;
pub(crate) mod response_adapter;

use anyhow::Result;

pub(crate) trait Adaptor {
    type From;
    type To;

    fn adapt(&self, from: Self::From) -> Result<Self::To> {
        self.before_adapt(&from);
        let to = self.do_adapt(from);
        self.after_adapt(to.as_ref());
        to
    }

    fn before_adapt(&self, from: &Self::From);

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>);

    fn do_adapt(&self, from: Self::From) -> Result<Self::To>;
}
