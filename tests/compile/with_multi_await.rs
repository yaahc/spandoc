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

    /// this shouldn't fail
    let _fut4 = async {
        let _ = fut2.await;
        fut3.await
    };

    let _fut5 = async {
        let fut = async {
            async {
                4
            }
        };

        /// this should totally work!
        let _four = fut.await.await;
    };

}
