#[spandoc::spandoc]
async fn test_thing() {
    /// SPANDOC: thing
    async {}.await
}

fn is_send<T: Send>(_t: T) {}

#[test]
fn test_is_send() {
    is_send(test_thing());
}
