//! Host thread stack for Hybrid Native v1 package typecheck/eval.
//!
//! Windows default stack is 1 MiB. Elaborated graphics/math synthetic bodies
//! recurse past that in Core infer/eval. CLI, GUI, and deep package tests use
//! [`HOST_STACK_SIZE`].

/// Stack size for host binaries that ingest package-shaped `.rpx`.
pub const HOST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Run `f` on a dedicated thread with [`HOST_STACK_SIZE`].
pub fn run_on_host_stack<T, F>(name: &'static str, f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(HOST_STACK_SIZE)
        .spawn(f)
        .map_err(|e| format!("spawn `{name}`: {e}"))?
        .join()
        .map_err(|_| format!("`{name}` panicked"))
}

#[cfg(test)]
mod tests {
    use super::{run_on_host_stack, HOST_STACK_SIZE};

    #[test]
    fn host_stack_runs_closure() {
        let n = run_on_host_stack("host-stack-test", || 7).expect("join");
        assert_eq!(n, 7);
        assert_eq!(HOST_STACK_SIZE, 8 * 1024 * 1024);
    }
}
