use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use tokio::time::timeout;

struct Fut<'a>(&'a dyn Fn() -> bool);

impl<'a> Future for Fut<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

async fn deadline_inner<F: Fn() -> bool + 'static>(
    wait_limit: Duration,
    condition: F,
) -> Result<(), tokio::time::error::Elapsed> {
    let fut = Fut(&condition);

    timeout(wait_limit, fut).await
}

/// Requires a condition closure to return `true` before the specified duration has elapsed.
#[macro_export]
macro_rules! deadline {
    ($wait_limit: expr, $condition: expr) => {{
        assert!(
            $crate::deadline_inner($wait_limit, $condition)
                .await
                .is_ok(),
            "the deadline has elapsed for condition: {}",
            stringify!($condition)
        );
    }};
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "the deadline has elapsed for condition: move || x == y")]
    async fn it_times_out() {
        let x = 1;
        let y = 2;

        deadline!(Duration::from_millis(1), move || x == y);
    }

    #[tokio::test]
    async fn it_waits_until_true() {
        let x = Arc::new(AtomicI32::new(41));
        let y = 42;

        let x_clone = x.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            x_clone.fetch_add(1, Ordering::SeqCst);
        });

        deadline!(Duration::from_millis(10), move || {
            x.load(Ordering::Relaxed) == y
        });
    }
}
