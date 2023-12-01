use std::{ffi::OsString, sync::OnceLock, time::Duration};

use clap::Parser;
use color_eyre::eyre::{eyre, ContextCompat, Error};
use tokio::sync::mpsc;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler,
    service_control_handler::ServiceControlHandlerResult,
    service_dispatcher,
};

use crate::Proxy;

static SERVICE: OnceLock<Service> = OnceLock::new();

#[derive(Parser, Debug)]
pub(crate) struct Service {
    #[arg(short, long)]
    /// Name of the service
    name: String,

    /// Proxy arguments
    #[command(flatten)]
    proxy: Proxy,
}

impl Service {
    pub(crate) fn run(self) -> Result<(), Error> {
        let name = self.name.clone();

        SERVICE
            .set(self)
            .map_err(|_| eyre!("Did not expect service to be registered"))?;

        service_dispatcher::start(name, ffi_service_main)?;

        Ok(())
    }
}

define_windows_service!(ffi_service_main, service_main);

fn service_run() -> Result<(), Error> {
    let service = SERVICE.get().context("Service is not registered")?;
    let (tx, rx) = mpsc::channel(1);
    let event_handler = move |event| match event {
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        ServiceControl::Stop => {
            let _ = tx.blocking_send(());

            ServiceControlHandlerResult::NoError
        }
        _ => ServiceControlHandlerResult::NotImplemented,
    };

    let status_handle = service_control_handler::register(&service.name, event_handler)?;

    // Set running status
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Run proxy
    run_proxy(service.proxy.clone(), rx)?;

    // Set stopped status
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn run_proxy(proxy: Proxy, mut rx: mpsc::Receiver<()>) -> Result<(), Error> {
    tokio::select! {
        res = proxy.run() => res,
        _ = rx.recv() => Ok(()),
    }
}

fn service_main(_: Vec<OsString>) {
    if let Err(err) = service_run() {
        log::error!("Error in service: {}", err);
    }
}
