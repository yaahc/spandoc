#![feature(backtrace)]

use spandoc::spandoc;
use tracing::instrument;
use tracing_error::{ErrorLayer, SpanTrace};
use tracing_subscriber::{layer::Layer, registry::Registry};

#[spandoc]
fn spanned() -> SpanTrace {
    /// Doing first grab of context
    let _a = get_context();

    /// Getting context from inside a block
    {
        get_context()
    }
}

#[spandoc]
#[instrument]
fn get_context() -> SpanTrace {
    /// Capturing context
    SpanTrace::capture()
}

fn main() {
    let subscriber = ErrorLayer::default().with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    let ctx = spanned();

    println!("Context:");
    println!("{}", ctx);

    println!("Backtrace:");
    println!("{}", std::backtrace::Backtrace::force_capture());
}
