pub mod scan;
pub mod space;
pub mod movefile;
pub mod planner;

pub use scan::FileEntry;
pub use planner::{SwapConfig, SwapReport, FeasibilityInfo, feasibility_check, run_dry, run_real};
