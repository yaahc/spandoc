# Spandoc

This project is still a WIP and is currently pre-release.

If you want to try using it check out the example and the dev dependencies for an idea how to setup spandoc + tracing.

If you run into broken line numbers its probably because of this issue
https://github.com/rust-lang/rust/issues/43081, `#[instrument]` and
`#[spandoc]` in particiular don't yet get along well. I have observed that
putting the spandoc attribute above the tracing instrument one can sometimes
fix broken line numbers. Proc macros /shrug.
