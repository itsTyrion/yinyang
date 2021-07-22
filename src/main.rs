#![windows_subsystem = "windows"]

use core::mem::MaybeUninit;
use trayicon::{MenuBuilder, MenuItem, TrayIconBuilder};
use winapi::um::winuser;

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

fn main() {
    tray();
}

fn switch_theme(change_system_theme: bool) -> std::io::Result<()> {
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

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let icon = include_bytes!("../assets/tray_icon.ico");

    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(icon)
        .tooltip("Light/Dark mode toggle")
        .on_click(Events::ClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .checkable("System Theme", false, CheckItem)
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
        r.iter().for_each(|m| match m {
            ClickTrayIcon => {
                let state = tray_icon.get_menu_item_checkable(CheckItem).unwrap();
                std::thread::spawn(move || switch_theme(state).unwrap());
            }
            CheckItem => {
                let state = tray_icon.get_menu_item_checkable(CheckItem).unwrap();
                tray_icon
                    .set_menu_item_checkable(CheckItem, !state)
                    .unwrap();
            }
            Events::Exit => {
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
