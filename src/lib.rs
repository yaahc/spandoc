pub use spandoc_attribute::spandoc;
use std::cell::Cell;
use tracing_futures::Instrument;

#[doc(hidden)]
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
