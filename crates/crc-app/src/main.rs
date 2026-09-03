use crc_app::{App, Session};

use std::path::PathBuf;

use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(windows)]
fn speak_utf8() {
    use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};

    const UTF8: u32 = 65001;
    unsafe {
        SetConsoleOutputCP(UTF8);
        SetConsoleCP(UTF8);
    }
}

#[cfg(not(windows))]
fn speak_utf8() {}

fn main() -> anyhow::Result<()> {
    speak_utf8();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,crc=info".into()),
        )
        .init();

    let mut smoke = false;
    let mut root = PathBuf::from(".");
    for argument in std::env::args().skip(1) {
        if argument == "--smoke" {
            smoke = true;
        } else {
            root = PathBuf::from(argument);
        }
    }

    let session = Session::open(&root)?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(session, smoke);
    event_loop.run_app(&mut app)?;
    Ok(())
}
