use spandoc::spandoc;

#[spandoc]
fn spanned() {
    /// Setting a to 1
    let _a = 1;
}

#[test]
fn happy() {
    spanned();
}
