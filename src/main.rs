#![windows_subsystem = "windows"]


use tray_icon::{
    menu::{Menu, MenuEvent, MenuItemBuilder}, Icon, TrayIcon, TrayIconBuilder
};
#[cfg(target_os = "windows")]
use winapi::{shared::windef::HWND, um::shellapi::{SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND}};

use winit::event_loop;

fn get_icon() -> Icon {
    let bytes = include_bytes!("icon.png");
    let image = image::load_from_memory(bytes).unwrap();
    let rgba = image.as_rgba8().unwrap();
    Icon::from_rgba(rgba.to_vec(), image.width(), image.height()).unwrap()
}

fn main() {
    let event_loop = event_loop::EventLoop::builder().build().unwrap();

    let mut icon: Option<TrayIcon> = None;
    let menu = Menu::new();
    menu.append(
        &MenuItemBuilder::new()
            .enabled(true)
            .id("open".into())
            .text("Open")
            .build(),
    )
    .unwrap();
    menu.append(
        &MenuItemBuilder::new()
            .enabled(true)
            .id("empty".into())
            .text("Empty")
            .build(),
    )
    .unwrap();
    menu.append(
        &MenuItemBuilder::new()
            .enabled(true)
            .id("exit".into())
            .text("Exit")
            .build(),
    )
    .unwrap();

    let menu_channel = MenuEvent::receiver();

    #[allow(deprecated)]
    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(16),
            ));

            #[cfg(not(target_os = "linux"))]
            if let winit::event::Event::NewEvents(winit::event::StartCause::Init) = event {
                icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(menu.clone()))
                        .with_title("Recycle Bin")
                        .with_icon(get_icon())
                        .build()
                        .unwrap(),
                )
            }

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == "exit" {
                    icon.take();
                    event_loop.exit();
                }

                if event.id == "open" {
                    #[cfg(target_os = "windows")]
                    std::process::Command::new("explorer")
                        .arg("shell:RecycleBinFolder")
                        .spawn()
                        .unwrap();

                    #[cfg(target_os = "linux")]
                    std::process::Command::new("xdg-open")
                        .arg("~/.local/share/Trash")
                        .spawn()
                        .unwrap();
                }

                if event.id == "empty" {
                    #[cfg(target_os = "windows")]
                    unsafe {
                        winapi::um::shellapi::SHEmptyRecycleBinA(
                            std::ptr::null_mut() as HWND,
                            std::ptr::null(),
                            SHERB_NOCONFIRMATION | SHERB_NOSOUND | SHERB_NOPROGRESSUI,
                        );
                    };

                    #[cfg(target_os = "linux")]
                    std::process::Command::new("rm")
                        .arg("-rf")
                        .arg("~/.local/share/Trash")
                        .spawn()
                        .unwrap();
                }
            }
        })
        .unwrap();
}
