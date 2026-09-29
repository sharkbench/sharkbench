use std::cell::Cell;

/// Limits the number of benchmarks that are run.
/// Skipped benchmarks (e.g. with `--missing`) do not count.
pub struct BenchmarkLimit {
    remaining: Cell<Option<usize>>,
}

impl BenchmarkLimit {
    pub fn new(limit: Option<usize>) -> Self {
        BenchmarkLimit {
            remaining: Cell::new(limit),
        }
    }

    /// Returns true if no more benchmarks should be run.
    pub fn reached(&self) -> bool {
        self.remaining.get() == Some(0)
    }

    /// Records that a benchmark has been run.
    pub fn record(&self) {
        if let Some(remaining) = self.remaining.get() {
            let remaining = remaining.saturating_sub(1);
            self.remaining.set(Some(remaining));
            if remaining == 0 {
                println!(" -> Benchmark limit reached");
            }
        }
    }
}
