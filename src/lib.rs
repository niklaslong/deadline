//! A one-macro crate to ensure assertions meet their deadlines.

use core::{
    future::{poll_fn, Future},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use tokio::time::{error::Elapsed, timeout};

pub struct Fut<'a>(&'a dyn Fn() -> bool);

impl<'a> Future for Fut<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[doc(hidden)]
pub async fn deadline_inner<F: Fn() -> bool + 'static>(
    wait_limit: Duration,
    condition: F,
) -> Result<(), Elapsed> {
    timeout(wait_limit, Fut(&condition)).await
}

/// Requires a condition closure to return `true` before the specified duration has elapsed.
///
/// This will panic if the provided closure doesn't evaluate to `true` before the provided duration
/// expires. Internally, it creates a [`Future`] from the closure that is polled until it returns
/// `true` or times out. This ensures the call is non-blocking to the async runtime.
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
/// deadline!(Duration::from_millis(10), move || {
///     x.load(Ordering::Relaxed) == y
/// });
/// # }
/// ```
#[macro_export]
macro_rules! deadline {
    ($wait_limit: expr, $condition: expr) => {{
        assert!(
            $crate::deadline_inner($wait_limit, $condition)
                .await
                .is_ok(),
            "the deadline has elapsed for condition: {}",
            {
                let normalized = stringify!($condition)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Some(wo_prefix) = normalized.strip_prefix("move || ") {
                    wo_prefix.to_owned()
                } else {
                    normalized
                }
            }
        );
    }};
}

/// The user never touches this; it's `doc(hidden)` so it
/// only shows up in the macro expansion.
#[doc(hidden)]
pub async fn new_deadline_inner<'a, F>(wait_limit: Duration, condition: F) -> Result<(), Elapsed>
where
    F: Fn() -> bool + 'a,
{
    timeout(
        wait_limit,
        poll_fn(move |cx: &mut Context<'_>| {
            if condition() {
                Poll::Ready(())
            } else {
                // re-schedule ourselves
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }),
    )
    .await
}

/// Ergonomic macro: no `move` required on the user’s end,
/// and we strip off the `||` for the panic message automatically.
#[macro_export]
macro_rules! new_deadline {
    ($wait_limit:expr, $cond:expr) => {{
        // create a zero-arg closure that borrows whatever `$cond` needs
        let __cond = || $cond;
        assert!(
            $crate::new_deadline_inner($wait_limit, __cond)
                .await
                .is_ok(),
            "the deadline has elapsed for condition: `{}`",
            stringify!($cond)
        );
    }};
}

/// Hidden helper which polls your zero-arg closure until it returns `true`,
/// re-waking the task each time it’s still `false`.
#[doc(hidden)]
pub async fn until_inner<'a, F>(condition: F)
where
    F: Fn() -> bool + 'a,
{
    poll_fn(move |cx: &mut Context<'_>| {
        if condition() {
            Poll::Ready(())
        } else {
            // schedule another poll immediately
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

/// Ergonomic macro: no `move ||` needed on the user side, and it
/// just awaits until the condition is true.
#[macro_export]
macro_rules! until {
    ($cond:expr) => {{
        // capture whatever `$cond` needs by reference
        let __cond = || $cond;
        // drive the hidden future to completion
        $crate::until_inner(__cond).await;
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

        deadline!(Duration::from_millis(1), move || x == y);
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

        deadline!(Duration::from_millis(10), move || {
            x.load(Ordering::Relaxed) == y
        });
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

        deadline!(Duration::from_millis(1000), move || {
            x.load(Ordering::Relaxed) == y
        });

        // Leave a bit of a gap, to avoid a flaky test.
        assert!(now.elapsed() < Duration::from_millis(10));
    }
}
