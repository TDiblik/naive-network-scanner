use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    Config,
};

#[cfg(debug_assertions)]
pub fn init() -> anyhow::Result<()> {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?;
    _ = log4rs::init_config(config)?;

    Ok(())
}

#[cfg(not(debug_assertions))]
pub fn init() -> anyhow::Result<()> {
    use log4rs::{
        append::{console::Target, file::FileAppender},
        encode::pattern::PatternEncoder,
        filter::threshold::ThresholdFilter,
    };

    let mut file_path = super::env::program_root_dir();
    file_path.push("log.txt");

    let config = Config::builder()
        .appender(
            Appender::builder().build(
                "logfile",
                Box::new(
                    FileAppender::builder()
                        .encoder(Box::new(PatternEncoder::new("{d} {l} {f} - {m}\n")))
                        .build(file_path)?,
                ),
            ),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Error)))
                .build(
                    "stderr",
                    Box::new(ConsoleAppender::builder().target(Target::Stderr).build()),
                ),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Warn),
        )?;

    _ = log4rs::init_config(config)?;

    Ok(())
}
