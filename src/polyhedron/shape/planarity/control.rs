/// Return if the expression is a break value, execute the provided statement
/// if it is a prune value.
/// https://github.com/petgraph/petgraph/blob/0.6.0/src/visit/dfsvisit.rs#L27
#[macro_export]
macro_rules! try_control {
    ($e:expr, $p:stmt) => {
        try_control!($e, $p, ());
    };
    ($e:expr, $p:stmt, $q:stmt) => {
        match $e {
            x => {
                if x.should_break() {
                    return x;
                } else if x.should_prune() {
                    $p
                } else {
                    $q
                }
            }
        }
    };
}

/// Control flow for `depth_first_search` callbacks.
#[derive(Copy, Clone, Debug)]
pub enum Control {
    /// Continue the DFS traversal as normal.
    Continue,
    /// Prune the current node from the DFS traversal. No more edges from this
    /// node will be reported to the callback. A `DfsEvent::Finish` for this
    /// node will still be reported. This can be returned in response to any
    /// `DfsEvent`, except `Finish`, which will panic.
    Prune,
    /// Stop the DFS traversal and return the provided value.
    Break,
}

/// Control flow for callbacks.
///
/// The empty return value `()` is equivalent to continue.
pub trait ControlFlow {
    fn continuing() -> Self;
    fn should_break(&self) -> bool;
    fn should_prune(&self) -> bool;
}

impl ControlFlow for () {
    fn continuing() {}
    #[inline]
    fn should_break(&self) -> bool {
        false
    }
    #[inline]
    fn should_prune(&self) -> bool {
        false
    }
}

impl ControlFlow for Control {
    fn continuing() -> Self {
        Control::Continue
    }
    fn should_break(&self) -> bool {
        if let Control::Break = *self {
            true
        } else {
            false
        }
    }
    fn should_prune(&self) -> bool {
        match *self {
            Control::Prune => true,
            Control::Continue | Control::Break => false,
        }
    }
}

impl<C: ControlFlow, E> ControlFlow for Result<C, E> {
    fn continuing() -> Self {
        Ok(C::continuing())
    }
    fn should_break(&self) -> bool {
        if let Ok(ref c) = *self {
            c.should_break()
        } else {
            true
        }
    }
    fn should_prune(&self) -> bool {
        if let Ok(ref c) = *self {
            c.should_prune()
        } else {
            false
        }
    }
}
