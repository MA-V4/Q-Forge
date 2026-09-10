pub mod html;
pub mod metrics;
pub mod runner;

pub use html::generate_report;
pub use metrics::{BenchmarkResult, BenchmarkSuite};
pub use runner::run_suite;
