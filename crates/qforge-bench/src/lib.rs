pub mod metrics;
pub mod runner;
pub mod html;

pub use metrics::{BenchmarkResult, BenchmarkSuite};
pub use runner::run_suite;
pub use html::generate_report;