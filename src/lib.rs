//! Attribute macro that transforms doc comments in functions into tracing
//! [`spans`](https://docs.rs/tracing/0.1.16/tracing/span/index.html).
//!
//! Spandoc provides a function level attribute macro that converts doc !
//! comments _within_ a function into tracing spans. These spans are entered
//! before the annotated expression and exited immediately after. Spandoc is also
//! async aware, so expressions that contain `.await`s will correctly instrument
//! the future, exit before yielding, and enter the span again when the future
//! resumes.
//!
//! # Details
//!
//! All doc comments intended to be transformed into spans **must** begin with `SPANDOC: `:
//!
//! ```rust
//! use spandoc::spandoc;
//! use tracing::info;
//!
//! #[spandoc]
//! fn foo() {
//!     /// SPANDOC: this will be converted into a span
//!     info!("event 1");
//!
//!     /// this will be ignored and produce a warning for an unused doc comment
//!     info!("event 2");
//! }
//! ```
//!
//! The spans that are created by spandoc are explicitly scoped to the expression
//! they're associated with.
//!
//! ```rust
//! use spandoc::spandoc;
//! use tracing::info;
//!
//! #[spandoc]
//! fn main() {
//!     tracing_subscriber::fmt::init();
//!     let local = 4;
//!
//!     /// SPANDOC: Emit a tracing info event {?local}
//!     info!("event 1");
//!
//!     info!("event 2");
//! }
//! ```
//!
//! Running the above example will produce the following output
//!
//! <pre><font color="#06989A"><b>spandoc</b></font> on <font color="#75507B"><b> await-support</b></font> <font color="#CC0000"><b>[!+] </b></font>is <font color="#FF8700"><b>📦 v0.1.3</b></font> via <font color="#CC0000"><b>🦀 v1.44.1</b></font>
//! <font color="#4E9A06"><b>❯</b></font> cargo run --example scoped
//! <font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.03s
//! <font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/scoped`
//! <font color="#A1A1A1">Jul 09 12:42:43.691 </font><font color="#4E9A06"> INFO</font> <b>main::comment{</b>local=4 text=Emit a tracing info event<b>}</b>: scoped: event 1
//! <font color="#A1A1A1">Jul 09 12:42:43.691 </font><font color="#4E9A06"> INFO</font> scoped: event 2</pre>
//!
//! Local variables can be associated with the generated spans by adding a
//! trailing block to the doc comment. The syntax for fields in the span is the
//! [same as in `tracing`](https://docs.rs/tracing/*/tracing/index.html#using-the-macros).
//!
//! ```rust
//! use spandoc::spandoc;
//! use tracing::info;
//!
//! #[spandoc]
//! fn foo() {
//!     let path = "fake.txt";
//!     /// SPANDOC: going to load config {?path}
//!     info!("event 1");
//!
//!     /// this will be ignored and produce a warning for an unused doc comment
//!     info!("event 2");
//! }
//! ```
//!
//! When applied to expressions that contain `await`s spandoc will correctly
//! use `instrument()` and exit/enter the span when suspending and resuming the
//! future. If there are multiple await expressions inside of the annotated
//! expression it will instrument each expression with the same span. The macro
//! will not recurse into `async` blocks.
//!
//!
//! ```rust
//! use std::future::Future;
//! use spandoc::spandoc;
//! use tracing::info;
//!
//! fn make_liz() -> impl Future<Output = Result<(), ()>> {
//!     info!("this will be printed in the span from `clever_girl`");
//!
//!     liz()
//! }
//!
//! async fn liz() -> Result<(), ()> {
//!     # let span = tracing::Span::current();
//!     # let metadata = span.metadata().expect("span should have metadata");
//!     # assert_eq!(metadata.name(), "clever_girl::comment");
//!     #
//!     info!("this will also be printed in the span from `clever_girl`");
//!
//!     // return a result so we can call map outside of the scope of the future
//!     Ok(())
//! }
//!
//! #[spandoc]
//! async fn clever_girl() {
//!     // This span will be entered before the await, exited correctly when the
//!     // future suspends, and instrument the future returned from `liz` with
//!     // `tracing-futures`
//!     /// SPANDOC: clever_girl async span
//!     make_liz().await.map(|()| info!("this will also be printed in the span"));
//! }
//! #
//! # tracing_subscriber::fmt::init();
//! # futures::executor::block_on(clever_girl());
//! ```
#![doc(html_root_url = "https://docs.rs/spandoc/0.2.0")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::missing_doc_code_examples,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![allow(clippy::needless_doctest_main)]
use std::sync::atomic::{AtomicBool, Ordering};

use tracing_futures::Instrument;

/// Attribute macro that transforms doc comments in functions into tracing [`spans`](https://docs.rs/tracing/0.1.16/tracing/span/index.html).
pub use spandoc_attribute::spandoc;

#[doc(hidden)]
pub struct FancyGuard<'a> {
    span: &'a tracing::Span,
    entered: AtomicBool,
}

impl<'a> FancyGuard<'a> {
    #[doc(hidden)]
    pub fn new(span: &'a tracing::Span) -> FancyGuard<'a> {
        span.with_subscriber(|(id, sub)| sub.enter(id));
        Self {
            span,
            entered: AtomicBool::new(true),
        }
    }

    #[doc(hidden)]
    pub async fn wrap<F>(&self, fut: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.span.with_subscriber(|(id, sub)| sub.exit(id));
        self.entered.store(false, Ordering::Relaxed);
        let output = fut.instrument(self.span.clone()).await;
        self.span.with_subscriber(|(id, sub)| sub.enter(id));
        self.entered.store(true, Ordering::Relaxed);
        output
    }
}

impl Drop for FancyGuard<'_> {
    fn drop(&mut self) {
        if self.entered.load(Ordering::Relaxed) {
            self.span.with_subscriber(|(id, sub)| sub.exit(id));
        }
    }
}
