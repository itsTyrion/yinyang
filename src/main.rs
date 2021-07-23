#![windows_subsystem = "windows"]

use core::mem::MaybeUninit;
use trayicon::{MenuBuilder, MenuItem, TrayIconBuilder};
use winapi::um::winuser;

use std::io::Result;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

fn main() {
    tray();
}

fn switch_theme(change_system_theme: bool) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let themes = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes")?;
    let personalize = themes.open_subkey_with_flags("Personalize", KEY_READ | KEY_SET_VALUE)?;

    let app_theme: u32 = personalize.get_value("AppsUseLightTheme")?;
    let app_theme_new = if app_theme == 0 { &1u32 } else { &0u32 };
    personalize.set_value("AppsUseLightTheme", app_theme_new)?;

    if change_system_theme {
        let system_theme: u32 = personalize.get_value("SystemUsesLightTheme")?;
        let system_theme_new = if system_theme == 0 { &1u32 } else { &0u32 };
        personalize.set_value("SystemUsesLightTheme", system_theme_new)?;
    }

    Ok(())
}

fn tray() {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        ClickTrayIcon,
        CheckItem,
        Exit,
        None,
    }

    use Events::*;

    let (sender, receiver) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("../assets/tray_icon.ico");

    let mut tray_icon = TrayIconBuilder::new()
        .sender(sender)
        .icon_from_buffer(icon)
        .tooltip("Light/Dark mode toggle")
        .on_click(ClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .checkable("System Theme", read_setting(), CheckItem)
                .separator()
                .with(MenuItem::Item {
                    name: "2021 itsTyrion".into(),
                    disabled: true, // Doesn't need to be clickcable
                    id: None,
                    icon: Option::None,
                })
                .separator()
                .item("Exit", Exit),
        )
        .build()
        .unwrap();

    std::thread::spawn(move || {
        receiver.iter().for_each(|m| match m {
            ClickTrayIcon => {
                let state = tray_icon.get_menu_item_checkable(CheckItem).unwrap();
                switch_theme(state).unwrap();
            }
            CheckItem => {
                let state = tray_icon.get_menu_item_checkable(CheckItem).unwrap();
                tray_icon
                    .set_menu_item_checkable(CheckItem, !state)
                    .unwrap();
                write_setting(!state);
            }
            Exit => {
                std::process::exit(0);
            }
            e => {
                println!("{:?}", e);
            }
        })
    });

    unsafe {
        // application message loop
        loop {
            let mut msg = MaybeUninit::uninit();
            let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }
}

use dirs::config_dir;
use std::fs;

fn write_setting(change_system_theme: bool) {
    let path = config_dir().unwrap().join("YinYang");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    let config = path.join("config.txt");

    fs::write(config, change_system_theme.to_string()).unwrap();
}

fn read_setting() -> bool {
    let config = config_dir().unwrap().join("YinYang").join("config.txt");
    if !config.exists() {
        write_setting(false);
    }
    let value = fs::read_to_string(config).unwrap();

    value.trim().parse().unwrap()
}
