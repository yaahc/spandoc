pub use spandoc_attribute::spandoc;
use std::cell::Cell;
use tracing_futures::Instrument;

pub struct FancyGuard<'a> {
    span: &'a tracing::Span,
    entered: Cell<bool>,
}

impl<'a> FancyGuard<'a> {
    pub fn new(span: &'a tracing::Span) -> FancyGuard<'a> {
        span.with_subscriber(|(id, sub)| sub.enter(id));
        Self {
            span,
            entered: Cell::new(true),
        }
    }

    pub async fn wrap<F>(&self, fut: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.span.with_subscriber(|(id, sub)| sub.exit(id));
        self.entered.set(false);
        let output = fut.instrument(self.span.clone()).await;
        self.span.with_subscriber(|(id, sub)| sub.enter(id));
        self.entered.set(true);
        output
    }
}

impl Drop for FancyGuard<'_> {
    fn drop(&mut self) {
        if self.entered.get() {
            self.span.with_subscriber(|(id, sub)| sub.exit(id));
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     async fn test() -> Result<(), ()> {
//         tracing::error!(
//             "event in wrapped future, shouldn't include fancy span and should include boo hoo"
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn it_works() {
//         tracing_subscriber::fmt::init();

//         let span = tracing::error_span!("fancy span");
//         tracing::error!("outside of fancy guard");
//         let guard = FancyGuard::new(&span);
//         tracing::error!("inside of fancy guard");

//         guard
//             .wrap({
//                 tracing::error!("this should happen inside of fancy guard");
//                 test()
//             })
//             .await
//             .map(|()| tracing::error!("back in fancy guard"))
//             .unwrap_err();
//     }
// }
