pub fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // Create a simple log format that contains code line numbers,
    // id and name of the thread that executes the program
    let format = tracing_subscriber::fmt::format()
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    // Create a rolling file appender that appends the app logs to a log file (base name: app.log)
    // This log file is being rotated on a daily basis
    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");

    // All the logs are written to a file by a separate a thread to avoid blocking the program
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Setup the Subscriber that collects the trace/log data
    tracing_subscriber::fmt()
        .event_format(format)
        .with_writer(non_blocking)
        .init();

    // The logs will not be captured if this guard variable is dropped
    // so return the Workerguard to the entry point of the program
    // that makes it live as long as the program (or main fn)
    guard
}
