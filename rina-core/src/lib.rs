pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
                .add_directive("rina=debug".parse().unwrap())
                .add_directive("rustls=off".parse().unwrap())
                .add_directive("hyper=off".parse().unwrap())
                .add_directive("h2=off".parse().unwrap())
                .add_directive("serenity=off".parse().unwrap())
                .add_directive("reqwest=off".parse().unwrap()),
        )
        .init();
}

pub mod agent;
pub mod attention;
pub mod character;
pub mod clients;
pub mod knowledge;
pub mod loaders;