#![allow(unused_doc_comments)]

use spandoc::spandoc;
use tracing::instrument;
use tracing_error::{Context, ErrorLayer};
use tracing_subscriber::{layer::Layer, registry::Registry, EnvFilter};

#[spandoc]
fn spanned() -> Context {
    /// Doing first grab of context
    let _a = get_context();

    /// Getting context from inside a block
    {
        get_context()
    }
}

#[instrument]
#[spandoc]
fn get_context() -> Context {
    /// Capturing context
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

    println!("Printing Context again");
    println!("Context:");
    println!("{}", ctx.span_backtrace());
}
