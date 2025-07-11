use tracing_subscriber::{fmt, EnvFilter};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::from_default_env()
        .add_directive("Organizer=debug".parse()?)
        .add_directive("iced=error".parse()?)
        .add_directive("wgpu_core=error".parse()?)
        .add_directive("wgpu_hal=error".parse()?);

    fmt().with_env_filter(filter).init();

    Ok(())
}
