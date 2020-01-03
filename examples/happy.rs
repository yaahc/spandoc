#![allow(unused_doc_comments)]

use spandoc::spandoc;
use tracing::instrument;
use tracing_error::{Context, ErrorLayer};
use tracing_subscriber::{layer::Layer, registry::Registry, EnvFilter};

#[spandoc]
fn spanned() -> Context {
    /// Setting a to 1
    let _a = eventful();

    /// Doing something in a block
    {
        /// Seriously, I'm gonna do it!
        eventful()
    }
}

#[instrument]
fn eventful() -> Context {
    /// so much to do, so little time
    let ctx = Context::current().unwrap();

    println!("Context:");
    println!("{}\n", ctx.span_backtrace());

    ctx
}

fn main() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = ErrorLayer::default()
        .and_then(filter)
        .with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    let ctx = spanned();

    println!("Context:");
    println!("{}", ctx.span_backtrace());
}
