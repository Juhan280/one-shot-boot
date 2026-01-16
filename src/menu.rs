use std::io::{self, Read};
use std::process;

use cursive::event::{Event, Key};
use cursive::views::{Dialog, OnEventView, SelectView};
use cursive::{Cursive, View};
use strum::{Display, VariantArray};

use crate::storage::{get_preferred_entry, set_preferred_entry};
use crate::system::{alloc_console, reboot, set_uefi_var};
use crate::utils::encode_utf16_null;
use crate::{ENTRIES, GUID, LOADER_ENTRY_ONE_SHOT};

#[derive(Debug)]
enum Action {
    Preferred { reboot: bool },
    Choose,
    UpdatePreferred,
    Exit,
}

#[derive(Copy, Clone, Debug, Display, VariantArray)]
enum Menu {
    #[strum(to_string = "Boot Preferred Now")]
    ChoosePreferredAndReboot,
    #[strum(to_string = "Boot Preferred Next Time")]
    ChoosePreferred,
    #[strum(to_string = "Select Other Entry")]
    ChooseOther,
    #[strum(to_string = "Configure Preferred Entry")]
    UpdatePreferred,
    #[strum(to_string = "Exit")]
    Exit,
}

impl From<Menu> for Action {
    fn from(value: Menu) -> Self {
        match value {
            Menu::ChoosePreferredAndReboot => Action::Preferred { reboot: true },
            Menu::ChoosePreferred => Action::Preferred { reboot: false },
            Menu::ChooseOther => Action::Choose,
            Menu::UpdatePreferred => Action::UpdatePreferred,
            Menu::Exit => Action::Exit,
        }
    }
}

trait WithHJKL {
    fn add_layer_with_hjkl<T: View>(&mut self, view: T);
}
impl WithHJKL for Cursive {
    fn add_layer_with_hjkl<T: View>(&mut self, view: T) {
        let view = OnEventView::new(view)
            .on_event('h', |s| s.on_event(Event::Key(Key::Left)))
            .on_event('j', |s| s.on_event(Event::Key(Key::Down)))
            .on_event('k', |s| s.on_event(Event::Key(Key::Up)))
            .on_event('l', |s| s.on_event(Event::Key(Key::Right)));

        self.add_layer(view);
    }
}

pub fn show_main_menu(s: &mut Cursive) {
    s.pop_layer();
    s.pop_layer();

    let select = SelectView::new()
        .with_all(Menu::VARIANTS.iter().map(|v| (v.to_string(), v)))
        .on_submit(show_submenu);

    s.add_layer_with_hjkl(Dialog::around(select).title("Main Menu"));
}

fn show_submenu(s: &mut Cursive, menu: &Menu) {
    let menu = *menu;

    match Action::from(menu) {
        Action::Preferred { reboot } => match get_preferred_entry() {
            Some(entry) => set_next_boot_entry(s, reboot, &entry),
            None => s.add_layer_with_hjkl(
                Dialog::text("No preferred entry set!!")
                    .button("Set Preferred", show_configure_prefered)
                    .button("Back", show_main_menu),
            ),
        },
        Action::Choose => {
            let select = SelectView::new()
                .with_all_str(ENTRIES.iter())
                .on_submit(move |s, entry| set_next_boot_entry(s, false, entry));

            s.add_layer_with_hjkl(
                Dialog::around(select)
                    .title(menu.to_string())
                    .button("Back", show_main_menu),
            );
        }
        Action::UpdatePreferred => show_configure_prefered(s),
        Action::Exit => {
            s.quit();
        }
    }
}

pub fn show_configure_prefered(s: &mut Cursive) {
    s.pop_layer();
    let select = SelectView::new()
        .with_all_str(ENTRIES.iter())
        .on_submit(move |s, entry| {
            s.pop_layer();

            set_preferred_entry(entry).unwrap();

            s.add_layer_with_hjkl(
                Dialog::text(format!("New preferred entry is set to: {entry}"))
                    .title("Preferred Entry Updated")
                    .button("Okay", show_main_menu),
            )
        });
    s.add_layer_with_hjkl(
        Dialog::around(select)
            .title(Menu::UpdatePreferred.to_string())
            .button("Back", show_main_menu),
    );
}

fn set_next_boot_entry(s: &mut Cursive, should_reboot: bool, entry: &str) {
    let data = encode_utf16_null(entry);
    if let Err(err) = set_uefi_var(LOADER_ENTRY_ONE_SHOT.into(), GUID.into(), None, &data) {
        alloc_console();
        eprintln!("{err}");
        io::stdin().read(&mut [0u8]).ok();
        process::exit(1);
    };

    if should_reboot {
        reboot();
    }

    s.add_layer_with_hjkl(
        Dialog::text(format!(
            "{LOADER_ENTRY_ONE_SHOT} variable is set to \"{entry}\""
        ))
        .button("Okay", show_main_menu)
        .button("Reboot", |_| reboot())
        .button("Exit", |_| process::exit(0)),
    );
}
