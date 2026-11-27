//! Platform-independent application engine for Lyreniq.
//! Coordinates streaming file analysis, CSV export, and (in P3) real-time live sessions.
#![forbid(unsafe_code)]

mod analysis;
mod error;
mod export;

pub use analysis::{
    AnalysisProgress, AnalysisSummary, FileAnalysisConfig, FileAnalysisResult, FileAnalyzer,
    TimelineFrame,
};
pub use error::EngineError;
pub use export::{export_timeline_csv, export_timeline_csv_to_path};
