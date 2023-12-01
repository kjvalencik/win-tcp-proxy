use clap::Parser;
use color_eyre::eyre::Error;

use crate::{install::Install, proxy::Proxy, service::Service, uninstall::Uninstall};

mod install;
mod proxy;
mod service;
mod uninstall;

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Args {
    /// Start a TCP proxy
    Proxy(Proxy),
    /// Install a TCP proxy service
    Install(Install),
    /// Uninstall a TCP proxy service
    Uninstall(Uninstall),
    /// Should only be executed by a service
    Service(Service),
}

fn main() -> Result<(), Error> {
    color_eyre::install()?;
    env_logger::try_init()?;

    match Args::parse() {
        Args::Proxy(proxy) => run_proxy(proxy),
        Args::Install(install) => install.run(),
        Args::Uninstall(uninstall) => uninstall.run(),
        Args::Service(service) => service.run(),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn run_proxy(proxy: Proxy) -> Result<(), Error> {
    proxy.run().await
}
