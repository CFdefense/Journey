use {
	crate::global::*,
	std::{
		fs::{self, File},
		io::{BufWriter, Write},
		path::Path,
		sync::{Once, OnceLock},
	},
	tracing::error,
	tracing_appender::{non_blocking::NonBlocking, rolling},
	tracing_subscriber::{
		EnvFilter, Layer, fmt::time::SystemTime, layer::SubscriberExt, util::SubscriberInitExt,
	},
};

static INIT_LOG: Once = Once::new();
static mut LOG_WRITER: OnceLock<NonBlocking> = OnceLock::new();
static mut TOOLS_LOG_WRITER: OnceLock<NonBlocking> = OnceLock::new();

/// When the program panics, the backtrace is outputted to `logs/crash.log`.
pub fn init_panic_handler() {
	unsafe {
		// Safety
		//
		// Always safe on Windows.
		//
		// Other platforms: risk of race condition in multi-threaded environment.
		// We are not reading/writing this environment variable from multiple threads, so we're good.
		std::env::set_var("RUST_BACKTRACE", "full");
	}
	std::panic::set_hook(Box::new(move |panic_info| {
		const WRITE_ERR: &str = "Could not write to crash log";
		error!("{}", panic_info);
		println!("{}", panic_info);

		fs::create_dir_all(LOG_DIR).expect("Could create crash log");
		let file = File::create(Path::new(LOG_DIR).join(CRASH_LOG))
			.expect("Could not create crash log file");
		let backtrace = std::backtrace::Backtrace::capture();
		let mut writer = BufWriter::new(file);

		writeln!(writer, "Time: {}", chrono::Local::now()).expect(WRITE_ERR);
		writeln!(writer, "{panic_info}").expect(WRITE_ERR);
		writeln!(writer, "stack backtrace:\n{backtrace}").expect(WRITE_ERR);
		writeln!(writer, "Process finished with exit code 101").expect(WRITE_ERR);
		writer.flush().expect(WRITE_ERR);
	}));
}

/// Creates a tracing registry and adds a layer to it. Layer outputs to `logs/latest.log`.
///
/// See `.env` variable `RUST_LOG` for layer filter. These variables should be loaded into the environment for the filter to work.
/// See [dotenvy].
pub fn init_logger() {
	INIT_LOG.call_once(|| {
		// Remove old logs
		_ = fs::remove_file(Path::new(LOG_DIR).join(LATEST_LOG));
		_ = fs::remove_file(Path::new(LOG_DIR).join(TOOLS_LOG));
		
		// Setup main log writer
		let (log_writer, log_guard) =
			tracing_appender::non_blocking(rolling::never(LOG_DIR, LATEST_LOG));
		let latest_log_layer = tracing_subscriber::fmt::layer()
			.with_timer(SystemTime)
			.with_ansi(false)
			.log_internal_errors(true)
			.with_target(true)
			.with_file(true)
			.with_line_number(true)
			.with_level(true)
			.with_thread_names(true)
			.with_thread_ids(true)
			.pretty()
			.with_writer(log_writer.clone())
			.with_filter(EnvFilter::from_default_env());
		
		// Setup tools log writer (only captures tool_trace target)
		let (tools_log_writer, tools_guard) =
			tracing_appender::non_blocking(rolling::never(LOG_DIR, TOOLS_LOG));
		let tools_log_layer = tracing_subscriber::fmt::layer()
			.with_timer(SystemTime)
			.with_ansi(false)
			.log_internal_errors(false)
			.with_target(false)
			.with_file(false)
			.with_line_number(false)
			.with_level(false)
			.with_thread_names(false)
			.with_thread_ids(false)
			.compact()
			.with_writer(tools_log_writer.clone())
			.with_filter(EnvFilter::new("tool_trace=info"));
		
		tracing_subscriber::registry()
			.with(latest_log_layer)
			.with(tools_log_layer)
			.init();

		#[allow(static_mut_refs)]
		unsafe {
			_ = LOG_WRITER.set(log_writer);
			_ = TOOLS_LOG_WRITER.set(tools_log_writer);
		}

		// Guards have to have a static lifetime.
		// We can just let the OS clean it up for us when the process is killed.
		Box::leak(Box::new(log_guard));
		Box::leak(Box::new(tools_guard));
	})
}

#[allow(unused)]
pub fn log_writer() -> &'static mut NonBlocking {
	#[allow(static_mut_refs)]
	unsafe {
		LOG_WRITER.get_mut().expect("Logger not initialized")
	}
}

/// Macro for logging tool calls to tools.log with a simple stack trace format
/// Usage: tool_trace!(agent: "orchestrator", tool: "route_task", status: "start", details: "task_type=research")
#[macro_export]
macro_rules! tool_trace {
	(agent: $agent:expr, tool: $tool:expr, status: $status:expr) => {
		tracing::info!(
			target: "tool_trace",
			"[{}] {} | {}",
			$agent,
			$tool,
			$status
		);
	};
	(agent: $agent:expr, tool: $tool:expr, status: $status:expr, details: $details:expr) => {
		tracing::info!(
			target: "tool_trace",
			"[{}] {} | {} | {}",
			$agent,
			$tool,
			$status,
			$details
		);
	};
}
