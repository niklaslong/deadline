//! A one-macro crate to ensure assertions meet their deadlines.

use core::time::Duration;

use tokio::time::{error::Elapsed, timeout};

#[doc(hidden)]
pub async fn deadline_inner<F>(wait_limit: Duration, mut condition: F) -> Result<(), Elapsed>
where
    F: FnMut() -> bool,
{
    timeout(wait_limit, async move {
        while !condition() {
            tokio::task::yield_now().await;
        }
    })
    .await
}

/// Requires a condition expression to evaluate to `true` before the specified duration has
/// elapsed.
///
/// This will panic if the provided expression doesn't evaluate to `true` before the provided
/// duration expires. Internally, the expression is re-checked after each cooperative yield to
/// the runtime, so state changes are caught with minimal latency without blocking other tasks.
///
/// # Examples
///
/// Waiting for an `AtomicI32` to be incremented to `42`:
///
/// ```rust
/// # #[tokio::main]
/// # async fn main() {
/// #     use std::{
/// #         sync::{
/// #             atomic::{AtomicI32, Ordering},
/// #             Arc,
/// #         },
/// #         time::Duration,
/// #     };
/// #
/// #     use deadline::deadline;
/// let x = Arc::new(AtomicI32::new(41));
/// let y = 42;
///
/// let x_clone = x.clone();
/// tokio::spawn(async move {
///     tokio::time::sleep(Duration::from_millis(5)).await;
///     x_clone.fetch_add(1, Ordering::SeqCst);
/// });
///
/// deadline!(Duration::from_millis(10), x.load(Ordering::Relaxed) == y);
/// # }
/// ```
#[macro_export]
macro_rules! deadline {
    ($wait_limit:expr, $condition:expr $(,)?) => {{
        assert!(
            $crate::deadline_inner($wait_limit, || $condition)
                .await
                .is_ok(),
            "the deadline has elapsed for condition: {}",
            stringify!($condition)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        );
    }};
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "the deadline has elapsed for condition: x == y")]
    async fn it_times_out() {
        let x = 1;
        let y = 2;

        deadline!(Duration::from_millis(1), x == y);
    }

    #[tokio::test]
    async fn it_waits_until_true() {
        let x = Arc::new(AtomicI32::new(41));
        let y = 42;

        let x_clone = x.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            x_clone.fetch_add(1, Ordering::SeqCst);
        });

        deadline!(Duration::from_millis(10), x.load(Ordering::Relaxed) == y);
    }

    #[tokio::test]
    async fn it_only_waits_as_long_as_needed() {
        let x = Arc::new(AtomicI32::new(41));
        let y = 42;

        let x_clone = x.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            x_clone.fetch_add(1, Ordering::SeqCst);
        });

        let now = std::time::Instant::now();

        deadline!(Duration::from_millis(1000), x.load(Ordering::Relaxed) == y);

        // Leave a bit of a gap, to avoid a flaky test.
        assert!(now.elapsed() < Duration::from_millis(10));
    }
}
