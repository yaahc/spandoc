fn blah() {
    let val = foo_fut()
        .await
        .map_err()
        .to_other_fut()
        .await
        .map(bar.await);
}

async fn wrap(&self, f: impl Future) {
    self.span.with_subscriber(|(id, sub)| sub.exit(id));
    f.instrument(self.span).await;
    self.span.with_subscriber(|(id, sub)| sub.enter(id));
}

// --- fancy_guard version courtesy of Nika ---
async fn blah() {
    let span = span!(); // create the span & stuff.
    let fancy_guard = FancyGuard::new(&span);
    let val =
        fancy_guard.wrap(
            fancy_guard.enter_after(fancy_guard.wrap(foo_fut())
                .await)
                .map_err()
                .to_other_fut()
        )
        .await
        .map(fancy_guard.wrap(bar).await);
}

// --- split on method call version ---
fn blah() {
    let val = {
        // recurse
        let fut = foo_fut().await.map_err().to_other_fut();
        // instrument
        let val = fut.await;
        // recurse
        let arg1 = bar.await;
        // enter
        val.map(arg1)
    };
}

fn blah() {
    let val = {
        let fut = {
            // enter
            let fut = foo_fut();
            // instrument
            let val = fut.await;
            // enter
            val.map_err().to_other_fut()
        };
        // instrument
        let val = fut.await;
        // recurse
        let arg1 = bar.await;
        // enter
        val.map(arg1)
    };
}

fn blah() {
    let val = {
        let fut = {
            // enter
            let fut = foo_fut();
            // instrument
            let val = fut.await;
            // enter
            val.map_err().to_other_fut()
        };
        // instrument
        let val = fut.await;
        // instrument
        let arg1 = bar.await;
        // enter
        val.map(arg1)
    };
}

fn blah_equivalent() {
    let val = {
        let fut = foo_fut();
        let val = fut.await;
        let fut = val.map_err().to_other_fut();
        let val = fut.await;
        val.map()
    };
}
