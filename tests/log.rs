#[cfg(test)]
mod tests {
	use {
		Capping2025::{constants::*, log},
		std::{fs, io::Write, path::Path, time::Duration},
		tracing::{error, info, trace}
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
		std::panic::catch_unwind(||{
			panic!("Test panic");
		}).unwrap_err();
		assert!(fs::read_to_string(CRASH_LOG).unwrap().len() > 0);
	}
}