use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, serde::Serialize)]
pub struct GameStatePayload {
    pub running: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct ProcessInfoPayload {
    pub running: bool,
    pub process_name: String,
}

fn check_game_running() -> bool {
    let mut sys = SYSTEM.lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let is_running = sys
        .processes_by_name(OsStr::new("Among Us.exe"))
        .next()
        .is_some();
    drop(sys);
    is_running
}

#[cfg(target_os = "windows")]
mod windows_wmi {
    use super::*;
    use std::sync::mpsc as std_mpsc;

    pub fn monitor_wmi_events(
        app: AppHandle,
        shutdown_rx: std_mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut was_running = false;

        loop {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            let is_running = check_game_running();

            if is_running != was_running {
                let _ = app.emit(
                    "game-state-changed",
                    GameStatePayload {
                        running: is_running,
                    },
                );
                if is_running {
                    let _ = app.emit(
                        "process-created",
                        ProcessInfoPayload {
                            running: true,
                            process_name: "Among Us.exe".to_string(),
                        },
                    );
                } else {
                    let _ = app.emit(
                        "process-deleted",
                        ProcessInfoPayload {
                            running: false,
                            process_name: "Among Us.exe".to_string(),
                        },
                    );
                }
                was_running = is_running;
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }
}

pub fn start_game_monitor(app: AppHandle) {
    if MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        use std::sync::mpsc as std_mpsc;
        let (_shutdown_tx, shutdown_rx) = std_mpsc::channel::<()>();

        std::thread::spawn(move || {
            if let Err(e) = windows_wmi::monitor_wmi_events(app, shutdown_rx) {
                log::error!("WMI monitor error: {e}");
            }
            MONITOR_RUNNING.store(false, Ordering::SeqCst);
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::sync::mpsc as std_mpsc;
        let (_shutdown_tx, shutdown_rx) = std_mpsc::channel::<()>();

        std::thread::spawn(move || {
            let mut was_running = false;

            loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                let is_running = check_game_running();

                if is_running != was_running {
                    let _ = app.emit(
                        "game-state-changed",
                        GameStatePayload {
                            running: is_running,
                        },
                    );
                    if is_running {
                        let _ = app.emit(
                            "process-created",
                            ProcessInfoPayload {
                                running: true,
                                process_name: "Among Us.exe".to_string(),
                            },
                        );
                    } else {
                        let _ = app.emit(
                            "process-deleted",
                            ProcessInfoPayload {
                                running: false,
                                process_name: "Among Us.exe".to_string(),
                            },
                        );
                    }
                    was_running = is_running;
                }

                std::thread::sleep(Duration::from_secs(2));
            }

            MONITOR_RUNNING.store(false, Ordering::SeqCst);
        });
    }
}

pub fn stop_game_monitor() {
    MONITOR_RUNNING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn get_game_running() -> bool {
    check_game_running()
}

#[tauri::command]
pub fn stop_monitor() {
    stop_game_monitor()
}
