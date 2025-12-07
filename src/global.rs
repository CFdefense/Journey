pub const LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/logs");
pub const CRASH_LOG: &str = "crash.log";
pub const LATEST_LOG: &str = "latest.log";
pub const TOOLS_LOG: &str = "tools.log";
pub const DIST_DIR: &str = "frontend/dist";
pub const MESSAGE_PAGE_LEN: i32 = 10;
pub const EVENT_SEARCH_RESULT_LEN: i32 = 10;

#[cfg(test)]
pub const TEST_COOKIE_EXP_SECONDS: i64 = 60;
