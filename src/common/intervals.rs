use std::time::Duration;

const COMMON: u64 = 15;

// Worker related
pub const WORKER_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(COMMON);
pub const WORKER_FUZZDRIVER_STAT_UPLOAD_INTERVAL: Duration = Duration::from_secs(5);
pub const WORKER_TASK_REFRESH_INTERVAL: Duration = Duration::from_secs(COMMON);
pub const WORKER_PROCESS_CHECK_INTERVAL: Duration = Duration::from_secs(COMMON);

// Master related
pub const MASTER_SCHEDULER_INTERVAL: Duration = Duration::from_secs(COMMON);

