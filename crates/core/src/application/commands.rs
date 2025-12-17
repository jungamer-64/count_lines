mod ports;
mod run_analysis;

pub use crate::domain::model::MeasurementOutcome;
pub use ports::{
    AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
    SnapshotComparator,
};
pub use run_analysis::{RunAnalysisCommand, RunAnalysisHandler, RunOutcome};
