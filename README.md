## spandoc

[![Build Status][actions-badge]][actions-url]
[![Latest Version][version-badge]][version-url]
[![Rust Documentation][docs-badge]][docs-url]

[actions-badge]: https://github.com/yaahc/spandoc/workflows/Continuous%20integration/badge.svg
[actions-url]: https://github.com/yaahc/spandoc/actions?query=workflow%3A%22Continuous+integration%22
[version-badge]: https://img.shields.io/crates/v/spandoc.svg
[version-url]: https://crates.io/crates/spandoc
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg
[docs-url]: https://docs.rs/spandoc

Attribute macro that transforms doc comments in functions into tracing [`spans`](https://docs.rs/tracing/0.1.16/tracing/span/index.html).

# Details

All doc comments intended to be transformed into spans **must** begin with `SPANDOC: `:

```rust
use spandoc::spandoc;
use tracing::info;

#[spandoc]
fn foo() {
    /// SPANDOC: this will be converted into a span
    info!("event 1");

    /// this will be ignored and produce a warning for an unused doc comment
    info!("event 2");
}
```

The spans that are created by spandoc are explicitly scoped to the expression
they're associated with.

```rust
use spandoc::spandoc;
use tracing::info;

#[spandoc]
fn main() {
    tracing_subscriber::fmt::init();
    let local = 4;

    /// SPANDOC: Emit a tracing info event {?local}
    info!("event 1");

    info!("event 2");
}
```

Running the above example will produce the following output

<pre><font color="#06989A"><b>spandoc</b></font> on <font color="#75507B"><b> await-support</b></font> <font color="#CC0000"><b>[!+] </b></font>is <font color="#FF8700"><b>📦 v0.1.3</b></font> via <font color="#CC0000"><b>🦀 v1.44.1</b></font>
<font color="#4E9A06"><b>❯</b></font> cargo run --example scoped
<font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.03s
<font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/scoped`
<font color="#A1A1A1">Jul 09 12:42:43.691 </font><font color="#4E9A06"> INFO</font> <b>main::comment{</b>local=4 text=Emit a tracing info event<b>}</b>: scoped: event 1
<font color="#A1A1A1">Jul 09 12:42:43.691 </font><font color="#4E9A06"> INFO</font> scoped: event 2</pre>

Local variables can be associated with the generated spans by adding a
trailing block to the doc comment. The syntax for fields in the span is the
[same as in `tracing`](https://docs.rs/tracing/*/tracing/index.html#using-the-macros).

```rust
use spandoc::spandoc;
use tracing::info;

#[spandoc]
fn foo() {
    let path = "fake.txt";
    /// SPANDOC: going to load config {?path}
    info!("event 1");

    /// this will be ignored and produce a warning for an unused doc comment
    info!("event 2");
}
```

When applied to expressions that contain `await`s spandoc will correctly
use `instrument()` and exit/enter the span when suspending and resuming the
future. If there are multiple await expressions inside of the annotated
expression it will instrument each expression with the same span. The macro
will not recurse into `async` blocks.


```rust
use std::future::Future;
use spandoc::spandoc;
use tracing::info;

fn make_liz() -> impl Future<Output = Result<(), ()>> {
    info!("this will be printed in the span from `clever_girl`");

    liz()
}

async fn liz() -> Result<(), ()> {
    info!("this will also be printed in the span from `clever_girl`");

    // return a result so we can call map outside of the scope of the future
    Ok(())
}

async fn clever_girl() {
    // This span will be entered before the await, exited correctly when the
    // future suspends, and instrument the future returned from `liz` with
    // `tracing-futures`
    /// clever_girl async span
    make_liz().await.map(|()| info!("this will also be printed in the span"));
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
