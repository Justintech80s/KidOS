#[cfg(target_os = "windows")]
mod ipc_server;
#[cfg(target_os = "windows")]
mod windows_host {
    use guardian_service::windows_lockdown::{
        LockdownInspection, WindowsAssignedAccessAdapter, WindowsLockdownAdapter,
    };
    use std::{
        ffi::OsString,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    };
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    pub const SERVICE_NAME: &str = "KidOSGuardian";

    define_windows_service!(ffi_service_main, service_main);

    pub fn run() -> Result<(), windows_service::Error> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    fn service_main(_arguments: Vec<OsString>) {
        if let Err(error) = run_service() {
            eprintln!("KidOS Guardian service failed: {error}");
        }
    }

    fn run_service() -> Result<(), windows_service::Error> {
        let stopping = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stopping);

        let status_handle = service_control_handler::register(
            SERVICE_NAME,
            move |control| match control {
                ServiceControl::Stop => {
                    stop_flag.store(true, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            },
        )?;

        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        std::thread::spawn(|| ipc_server::run_pipe_server());

        let adapter = WindowsAssignedAccessAdapter::default();
        while !stopping.load(Ordering::SeqCst) {
            match adapter.inspect() {
                Ok(LockdownInspection::Configured | LockdownInspection::NotConfigured) => {}
                Ok(LockdownInspection::Unsupported) => {
                    eprintln!("KidOS Guardian: Assigned Access provider is unsupported.");
                }
                Err(error) => {
                    eprintln!("KidOS Guardian: lockdown provider inspection failed: {error:?}");
                }
            }

            for _ in 0..10 {
                if stopping.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }

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
}

#[cfg(target_os = "windows")]
fn main() {
    if let Err(error) = windows_host::run() {
        eprintln!("Unable to start KidOS Guardian as a Windows service: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("kidos-guardian-host is only supported on Windows.");
}
