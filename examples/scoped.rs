use spandoc::spandoc;
use tracing::info;

#[spandoc]
fn main() {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();
    let local = 4;

    /// SPANDOC: Emit a tracing info event {?bar}
    info!("event 1");

    info!("event 2");
}
