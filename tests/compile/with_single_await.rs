use spandoc::spandoc;

#[spandoc]
fn main() {
    let fut = async {
        4usize
    };

    let _fut2 = async {
        /// do the thing
        fut.await
    };
}
