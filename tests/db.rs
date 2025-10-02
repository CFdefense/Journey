use {
	Capping2025::db,
	tracing::info
};

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
			let _ = db::create_pool().await;
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

	let pool = db::create_pool().await;

	// Simple liveness query
	let row: (i32,) = sqlx::query_as("SELECT 1")
		.fetch_one(&pool)
		.await
		.expect("SELECT 1 should succeed");
	assert_eq!(row.0, 1);
}