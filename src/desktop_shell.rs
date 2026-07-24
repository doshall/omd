//! System tray and global shortcuts (desktop only).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellCommand {
    ShowWindow,
    NewFile,
    Quit,
}

#[cfg(feature = "desktop-shell")]
mod imp {
    use std::sync::mpsc::{self, Receiver};
    #[cfg(target_os = "linux")]
    use std::sync::mpsc::Sender;

    use global_hotkey::hotkey::{Code, HotKey, Modifiers};
    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
    #[cfg(target_os = "linux")]
    use gtk;

    use super::ShellCommand;

    pub struct DesktopShell {
        hotkey_manager: GlobalHotKeyManager,
        show_window_hotkey: HotKey,
        new_file_hotkey: HotKey,
        #[cfg(not(target_os = "linux"))]
        _tray_icon: TrayIcon,
        command_rx: Receiver<ShellCommand>,
    }

    impl DesktopShell {
        pub fn try_init(enable_tray: bool, register_hotkeys: bool) -> Option<Self> {
            if !enable_tray {
                return None;
            }
            let icon = app_icon()?;
            let (tx, command_rx) = mpsc::channel();

            let hotkey_manager = GlobalHotKeyManager::new().ok()?;
            let show_window_hotkey =
                HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO);
            let new_file_hotkey =
                HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyN);
            if register_hotkeys {
                hotkey_manager.register(show_window_hotkey).ok()?;
                hotkey_manager.register(new_file_hotkey).ok()?;
            }

            #[cfg(not(target_os = "linux"))]
            let tray_icon = build_tray(icon.clone())?;

            #[cfg(target_os = "linux")]
            spawn_linux_tray(icon, tx);

            Some(Self {
                hotkey_manager,
                show_window_hotkey,
                new_file_hotkey,
                #[cfg(not(target_os = "linux"))]
                _tray_icon: tray_icon,
                command_rx,
            })
        }

        pub fn poll_commands(&self) -> Vec<ShellCommand> {
            let mut commands = Vec::new();

            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.id == self.show_window_hotkey.id() {
                    commands.push(ShellCommand::ShowWindow);
                } else if event.id == self.new_file_hotkey.id() {
                    commands.push(ShellCommand::NewFile);
                }
            }

            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(cmd) = menu_id_to_command(event.id.as_ref()) {
                    commands.push(cmd);
                }
            }

            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                    commands.push(ShellCommand::ShowWindow);
                }
            }

            while let Ok(cmd) = self.command_rx.try_recv() {
                commands.push(cmd);
            }

            commands
        }
    }

    pub(crate) fn menu_id_to_command(id: &str) -> Option<ShellCommand> {
        match id {
            "show" => Some(ShellCommand::ShowWindow),
            "new" => Some(ShellCommand::NewFile),
            "quit" => Some(ShellCommand::Quit),
            _ => None,
        }
    }

    fn build_menu() -> Menu {
        let menu = Menu::new();
        let _ = menu.append(&MenuItem::with_id("show", "Show omd", true, None));
        let _ = menu.append(&MenuItem::with_id("new", "New Document", true, None));
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&MenuItem::with_id("quit", "Quit", true, None));
        menu
    }

    #[cfg(not(target_os = "linux"))]
    fn build_tray(icon: Icon) -> Option<TrayIcon> {
        TrayIconBuilder::new()
            .with_menu(Box::new(build_menu()))
            .with_tooltip("omd — Markdown Editor")
            .with_icon(icon)
            .build()
            .ok()
    }

    #[cfg(target_os = "linux")]
    fn spawn_linux_tray(icon: Icon, tx: Sender<ShellCommand>) {
        std::thread::spawn(move || {
            #[cfg(target_os = "linux")]
            if gtk::init().is_err() {
                return;
            }
            let menu = build_menu();
            let _tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("omd — Markdown Editor")
                .with_icon(icon)
                .build();

            let menu_tx = tx;
            std::thread::spawn(move || {
                while let Ok(event) = MenuEvent::receiver().recv() {
                    if let Some(cmd) = menu_id_to_command(event.id.as_ref()) {
                        let _ = menu_tx.send(cmd);
                    }
                }
            });

            gtk::main();
        });
    }

    pub(crate) fn app_icon() -> Option<Icon> {
        const SIZE: u32 = 32;
        let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let edge = x < 2 || y < 2 || x >= SIZE - 2 || y >= SIZE - 2;
                let (r, g, b, a) = if edge {
                    (11, 94, 215, 255)
                } else {
                    (13, 110, 253, 255)
                };
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }
        Icon::from_rgba(rgba, SIZE, SIZE).ok()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn app_icon_builds() {
            assert!(app_icon().is_some());
        }

        #[test]
        fn menu_ids_map_to_commands() {
            assert_eq!(menu_id_to_command("show"), Some(ShellCommand::ShowWindow));
            assert_eq!(menu_id_to_command("quit"), Some(ShellCommand::Quit));
        }
    }
}

#[cfg(feature = "desktop-shell")]
pub use imp::DesktopShell;

#[cfg(not(feature = "desktop-shell"))]
pub struct DesktopShell;

#[cfg(not(feature = "desktop-shell"))]
impl DesktopShell {
    pub fn try_init(_enable_tray: bool, _register_hotkeys: bool) -> Option<Self> {
        None
    }

    pub fn poll_commands(&self) -> Vec<ShellCommand> {
        Vec::new()
    }
}
