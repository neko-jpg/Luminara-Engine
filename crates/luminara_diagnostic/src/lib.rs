//! Luminara Diagnostic Tools
//!
//! This crate provides comprehensive diagnostic, benchmarking, and profiling tools
//! for the Luminara Engine. It supports the Pre-Editor Engine Audit phase by enabling:
//!
//! - Architecture analysis and dependency graph visualization
//! - Performance benchmarking with industry comparisons
//! - Profiling tools for CPU, GPU, and memory analysis
//! - Code coverage and quality metrics
//!
//! # Modules
//!
//! - `architecture`: Architecture analysis and dependency graph tools
//! - `benchmark`: Comprehensive benchmark suite
//! - `profiler`: Performance profiling utilities
//! - `coverage`: Code coverage analysis tools

pub mod architecture;
pub mod benchmark;
pub mod profiler;

/// Re-export commonly used types
pub use architecture::{ArchitectureAnalyzer, AnalysisReport};
pub use benchmark::{BenchmarkSuite, BenchmarkResult};
pub use profiler::{PerformanceProfiler, ProfilingReport, SystemProfiler, Bottleneck, BottleneckSeverity};

/// Common error type for diagnostic tools
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Analysis error: {0}")]
    Analysis(String),
    
    #[error("Benchmark error: {0}")]
    Benchmark(String),
    
    #[error("Profiling error: {0}")]
    Profiling(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DiagnosticError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_crate_loads() {
        // Basic smoke test to ensure the crate compiles and loads
        assert!(true);
    }
}
