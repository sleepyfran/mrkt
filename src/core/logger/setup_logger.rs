/// Sets up the global logger using the `fern` crate, formatting log messages
/// with timestamps, targets, and log levels.
pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let now = time::OffsetDateTime::now_utc();
            let formatted_time = now
                .format(
                    &time::format_description::parse(
                        "[year]-[month]-[day] [hour]:[minute]:[second]",
                    )
                    .unwrap(),
                )
                .unwrap();

            out.finish(format_args!(
                "{}[{}][{}] {}",
                formatted_time,
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
