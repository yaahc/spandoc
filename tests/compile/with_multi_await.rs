use spandoc::spandoc;

#[spandoc]
fn main() {
    let fut = async {
        4usize
    };

    let fut2 = async {
        /// do the thing
        fut.await
    };

    let fut3 = async {
        5usize
    };

    /// this should fail
    let fut4 = async {
        let _ = fut2.await;
        fut3.await
    };
}
