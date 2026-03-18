use std::sync::OnceLock;

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init_logging() {
    LOGGING_INIT.get_or_init(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,omnichat=debug"));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact()
            .init();
    });
}
