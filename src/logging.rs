use std::io;

pub fn setup_logging(verbosity: u64) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => base_config.level(log::LevelFilter::Info),

        1 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("tokio_core", log::LevelFilter::Info)
            .level_for("tokio_reactor", log::LevelFilter::Info)
            .level_for("tokio_proto", log::LevelFilter::Info)
            .level_for("tokio_io", log::LevelFilter::Info)
            .level_for("mio", log::LevelFilter::Info)
            .level_for("tarpc", log::LevelFilter::Info),

        2 => base_config
            .level(log::LevelFilter::Trace)
            .level_for("tokio_core", log::LevelFilter::Info)
            .level_for("tokio_reactor", log::LevelFilter::Info)
            .level_for("tokio_proto", log::LevelFilter::Info)
            .level_for("tokio_io", log::LevelFilter::Info)
            .level_for("mio", log::LevelFilter::Info)
            .level_for("tarpc", log::LevelFilter::Info),

        _3_or_more => base_config.level(log::LevelFilter::Trace),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        }).chain(fern::log_file("auth-service.log")?);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            // special format for debug messages coming from our own crate.
            if record.level() > log::LevelFilter::Info && record.target() == "auth-service" {
                out.finish(format_args!(
                    "---\nDEBUG: {}: {}\n---",
                    chrono::Local::now().format("%H:%M:%S"),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "[{}][{}][{}] {}",
                    chrono::Local::now().format("%H:%M"),
                    record.target(),
                    record.level(),
                    message
                ))
            }
        }).chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}
