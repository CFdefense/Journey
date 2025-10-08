use {
    crate::global::*,
    std::{
        fs::{self, File},
        io::{BufWriter, Write},
        path::Path,
        sync::OnceLock,
    },
    tracing::error,
    tracing_appender::{
        non_blocking::{NonBlocking, WorkerGuard},
        rolling,
    },
    tracing_subscriber::{
        EnvFilter, Layer, fmt::time::SystemTime, layer::SubscriberExt, util::SubscriberInitExt,
    },
};

/// A writer to the `logs/latest.log` file.
///
/// The writer is here so you can flush or other fine-tunings.
///
/// # Safety
/// It's always safe to mutate the writer. Rusts [OnceLock] API is just bad.
static mut LOG_WRITER: OnceLock<NonBlocking> = OnceLock::new();

/// A guard to the `logs/latest.log` file.
///
/// You cannot output to the file once the guard is dropped, so this should be static.
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// When the program panics, the backtrace is outputted to `logs/crash.log`
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
    std::panic::set_hook(Box::new(|panic_info| {
        const FS_ERR: &str = "Could create crash log";
        const WRITE_ERR: &str = "Could not write to crash log";
        const PANIC_MSG: &str = "Panic - see crash.log";
        error!("{}", PANIC_MSG);
        println!("{}", PANIC_MSG);

        let path = Path::new(CRASH_LOG);
        fs::create_dir_all(path.parent().expect(FS_ERR)).expect(FS_ERR);
        let file = File::create(path).expect("Could not create crash log file");
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
	_ = fs::remove_file(Path::new(LOG_DIR).join(LATEST_LOG));
    let (log_writer, log_guard) =
        tracing_appender::non_blocking(rolling::never(LOG_DIR, LATEST_LOG));
    unsafe {
        // Safety
        //
        // It is always safe to mutate the writer.
        // Rust's OnceLock API is just bad.
        #[allow(static_mut_refs)]
        LOG_WRITER.set(log_writer.clone()).unwrap();
    }
    LOG_GUARD.set(log_guard).unwrap();
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
        .with_writer(log_writer)
        .with_filter(EnvFilter::from_default_env());
    tracing_subscriber::registry().with(latest_log_layer).init();
}

/// Accesses [LOG_WRITER], which writes to `log/latest.log`.
///
/// # Panics
/// If the [LOG_WRITER] has not been initialized. It should be initialized by calling [init_logger].
pub fn get_writer_mut() -> &'static mut NonBlocking {
    unsafe {
        // Safety
        //
        // It is always safe to mutate the writer.
        // Rust's OnceLock API is just bad.
        #[allow(static_mut_refs)]
        LOG_WRITER.get_mut().unwrap()
    }
}
