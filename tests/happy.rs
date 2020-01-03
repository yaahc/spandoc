use spandoc::spandoc;
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::FmtLayer, layer::Layer, registry::Registry, EnvFilter};

#[spandoc]
fn spanned() {
    /// Setting a to 1
    let _a = eventful();
}

#[instrument]
fn eventful() -> i32 {
    /// so much to do, so little time
    info!("doing the thing!");
    42
}

#[test]
fn happy() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = ErrorLayer::default()
        .and_then(FmtLayer::builder().with_target(false).finish())
        .and_then(filter)
        .with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    spanned();
}
