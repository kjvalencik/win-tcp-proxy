use std::{env, ffi::OsString, iter};

use clap::Parser;
use color_eyre::eyre::Error;
use windows_service::{
    service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::proxy::Proxy;

#[derive(Parser, Debug)]
pub(crate) struct Install {
    #[arg(short, long)]
    /// Name of the service
    name: String,

    /// Proxy arguments
    #[command(flatten)]
    proxy: Proxy,
}

impl Install {
    pub(crate) fn run(self) -> Result<(), Error> {
        log::trace!("Service {self:#?}");

        let launch_arguments = iter::once(OsString::from("service"))
            .chain(env::args().skip(2).map(OsString::from))
            .collect::<Vec<_>>();

        let executable_path = std::env::current_exe()?;
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        let display_name = format!("TCP Proxy ({})", self.name).into();
        let service_info = ServiceInfo {
            name: self.name.into(),
            display_name,
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path,
            launch_arguments,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        let service = service_manager.create_service(
            &service_info,
            ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
        )?;

        service.set_description("TCP Proxy")?;
        service.start::<String>(&[])?;

        Ok(())
    }
}
