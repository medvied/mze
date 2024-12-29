use clap::Parser;

use mze::{app, container, renderer};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Create container (an existing container is opened otherwise)
    #[arg(long, default_value_t = false)]
    container_create: bool,
    /// Container type
    #[arg(long)]
    container_type: String,
    /// URI for Container::new()
    #[arg(long)]
    container_uri: String,
    /// Renderer type
    #[arg(long)]
    renderer_type: String,
    /// URI for Renderer::new()
    #[arg(long)]
    renderer_uri: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::init();

    let args = Args::parse();

    let container = container::new(&args.container_type, &args.container_uri)?;
    if args.container_create {
        container.create()?;
    }

    let mut renderer =
        renderer::new(&args.renderer_type, &args.renderer_uri, container)?;

    renderer.run()
}
