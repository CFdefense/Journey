use {
    crate::{global::*, log},
    std::{fs, io::Write, path::Path, time::Duration},
    tracing::{error, info, trace},
};

/// Verifies that `logs/latest.log` is created and written to from log events.
#[test]
fn test_logger() {
	//dotenv doesn't work in github actions bc .env is ignored
	unsafe {
		// Safety
		//
		// Always safe on Windows.
		//
		// Other platforms: risk of race condition in multi-threaded environment.
		// We are not reading/writing this environment variable from multiple threads, so we're good.
		std::env::set_var("RUST_LOG", "warn,Capping2025=debug");
	}
	let latest_log_path = Path::new(LOG_DIR).join(LATEST_LOG);
	_ = fs::remove_file(latest_log_path.as_path());
	log::init_logger();
	trace!("Test trace");
	error!("Test error");
	log::get_writer_mut().flush().unwrap();
	//wait for IO to finish because flushing doesn't work?
	std::thread::sleep(Duration::from_millis(10));
	let logs = fs::read_to_string(latest_log_path).unwrap();
	info!("{logs}");
	assert!(logs.len() > 0);
}

/// Verifies that `logs/crash.log` is created and written to on a panic.
#[test]
fn test_panic_handler() {
    _ = fs::remove_file(CRASH_LOG);
    log::init_panic_handler();
    std::panic::catch_unwind(|| {
        panic!("Test panic");
    })
    .unwrap_err();
    assert!(fs::read_to_string(CRASH_LOG).unwrap().len() > 0);
}


/// Verifies that `db::create_pool` panics when `DATABASE_URL` is not set.
#[test]
fn test_db_pool_panics_without_env() {
	// Save and clear DATABASE_URL
	let prev = std::env::var("DATABASE_URL").ok();
	unsafe { std::env::remove_var("DATABASE_URL"); }

	let result = std::panic::catch_unwind(||{
		let rt = tokio::runtime::Runtime::new().unwrap();
		rt.block_on(async {
			// Should panic due to missing env var
			let _ = crate::db::create_pool().await;
		});
	});

	// Restore DATABASE_URL
	match prev {
		Some(val) => unsafe { std::env::set_var("DATABASE_URL", val) },
		None => unsafe { std::env::remove_var("DATABASE_URL") },
	}

	assert!(result.is_err());
}

/// Optional integration test requiring a real database in `DATABASE_URL`.
/// Run with: `cargo test -- --ignored`
#[tokio::test]
#[ignore]
async fn test_db_pool_connects_and_selects() {
	let database_url = match std::env::var("DATABASE_URL") {
		Ok(v) => v,
		Err(_) => {
			// Not set in most environments; mark as success skip
			info!("DATABASE_URL not set; skipping real DB test");
			return;
		}
	};

	// Ensure env var is present for this test
	unsafe { std::env::set_var("DATABASE_URL", database_url); }

	let pool = crate::db::create_pool().await;

	// Simple liveness query
	let row: (i32,) = sqlx::query_as("SELECT 1")
		.fetch_one(&pool)
		.await
		.expect("SELECT 1 should succeed");
	assert_eq!(row.0, 1);
}
