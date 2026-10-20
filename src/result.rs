//! Typed route result passing (`push_for_result`).
//!
//! Allows a screen to open a modal, sub-screen, or picker and asynchronously await
//! its return value via a type-safe oneshot channel.
//!
//! # Cancellation Guarantee
//! If the opened screen is popped or dismissed without explicitly completing the result,
//! the sender is dropped and the receiver returns `Err(NavigationResultError::Cancelled)`.

use std::any::Any;
use tokio::sync::oneshot;

/// Error returned when awaiting a route result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationResultError {
    /// The screen was closed, popped, or dismissed without sending a result.
    Cancelled,
}

impl std::fmt::Display for NavigationResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "Navigation cancelled: the screen was closed without producing a result"),
        }
    }
}

impl std::error::Error for NavigationResultError {}

/// Trait implemented by route configurations that return a typed value upon completion.
pub trait ResultRouteConfig: 'static {
    /// The type of data returned when this route completes.
    type Result: Send + 'static;
}

/// A type-erased handle for completing a route result from within the opened screen.
pub trait AnyResultSender: Send + 'static {
    /// Delivers the result to the awaiting caller and completes the transition.
    fn send_any(&mut self, result: Box<dyn Any + Send>) -> bool;

    /// Returns `true` if the awaiting receiver has already dropped.
    fn is_cancelled(&self) -> bool;
}

/// A strongly-typed sender given to a target screen to fulfill its result promise.
pub struct ResultSender<R: Send + 'static> {
    tx: Option<oneshot::Sender<R>>,
}

impl<R: Send + 'static> ResultSender<R> {
    /// Creates a new `ResultSender` wrapping an asynchronous oneshot sender.
    pub fn new(tx: oneshot::Sender<R>) -> Self {
        Self { tx: Some(tx) }
    }

    /// Sends the result to the caller.
    ///
    /// Returns `Ok(())` if the result was received, or `Err(R)` if the caller dropped the receiver.
    pub fn send(&mut self, value: R) -> Result<(), R> {
        if let Some(tx) = self.tx.take() {
            tx.send(value)
        } else {
            Err(value)
        }
    }

    /// Returns `true` if the awaiting receiver has already dropped or if the result was already sent.
    pub fn is_closed(&self) -> bool {
        self.tx.as_ref().map_or(true, |tx| tx.is_closed())
    }
}

impl<R: Send + 'static> AnyResultSender for ResultSender<R> {
    fn send_any(&mut self, result: Box<dyn Any + Send>) -> bool {
        if let Ok(value) = result.downcast::<R>() {
            self.send(*value).is_ok()
        } else {
            false
        }
    }

    fn is_cancelled(&self) -> bool {
        self.tx.as_ref().map_or(true, |tx| tx.is_closed())
    }
}
