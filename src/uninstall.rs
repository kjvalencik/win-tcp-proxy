use std::ffi::OsString;

use windows_service::{
    service::{ServiceAccess, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use clap::Parser;
use color_eyre::eyre::Error;

#[derive(Parser, Debug)]
pub(crate) struct Uninstall {
    #[arg(short, long)]
    /// Name of the service
    name: OsString,
}

impl Uninstall {
    pub(crate) fn run(self) -> Result<(), Error> {
        log::trace!("Service {self:#?}");

        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;

        let service = service_manager.open_service(&self.name, service_access)?;

        service.delete()?;

        // Try to stop
        if service.query_status()?.current_state != ServiceState::Stopped {
            service.stop()?;
        }

        Ok(())
    }
}
