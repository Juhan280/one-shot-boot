#![windows_subsystem = "windows"]
#![allow(clippy::unused_io_amount)]

mod menu;
mod storage;
mod system;
mod utils;

use std::io::{self, Read};
use std::sync::LazyLock;
use std::{panic, process};

use cursive::theme::Theme;

use crate::menu::{show_configure_prefered, show_main_menu};
use crate::storage::get_preferred_entry;
use crate::system::{alloc_console, get_uefi_var, reboot, set_uefi_var};
use crate::utils::{encode_utf16_null, parse_multi_sz};

const LOADER_ENTRY_ONE_SHOT: &str = "LoaderEntryOneShot";
// const LOADER_ENTRY_DEFAULT: &str = "LoaderEntryDefault";
const LOADER_ENTRIES: &str = "LoaderEntries";
const GUID: &str = "{4a67b082-0a4c-41cf-b6c7-440b29bb8c4f}";

static ENTRIES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let (_, data) = get_uefi_var(LOADER_ENTRIES.into(), GUID.into()).unwrap_or_else(|e| {
        alloc_console();
        eprintln!("{e}");
        io::stdin().read(&mut [0u8]).ok();
        process::exit(1);
    });

    parse_multi_sz(&data)
});

fn main() {
    panic::set_hook(Box::new(|info| {
        alloc_console();

        eprintln!("--- APPLICATION PANIC ---");
        eprintln!("{info}");

        eprintln!("\nPress Enter to close this window...");
        io::stdin().read(&mut [0u8]).ok();
    }));

    run().unwrap_or_else(|e| {
        alloc_console();
        eprintln!("Error occured: {e}");
        io::stdin().read(&mut [0u8]).ok();
    });
}

fn run() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    system::update_priviliges()?;

    if ENTRIES.is_empty() {
        alloc_console();
        eprintln!("{LOADER_ENTRIES} uefi variable has no entries!");
        io::stdin().read(&mut [0u8]).ok();
        process::exit(1);
    }

    let use_preferred = args.iter().any(|s| s == "--use-preferred");

    if use_preferred
        && let Some(preferred_entry) = get_preferred_entry()
        && ENTRIES.contains(&preferred_entry)
    {
        let data = encode_utf16_null(&preferred_entry);
        if let Err(err) = set_uefi_var(LOADER_ENTRY_ONE_SHOT.into(), GUID.into(), None, &data) {
            alloc_console();
            eprintln!(r#"Failed to set uefi variable "{LOADER_ENTRY_ONE_SHOT}""#);
            eprintln!("Error: {err}");
            io::stdin().read(&mut [0u8]).ok();
            process::exit(1);
        };
        reboot();
    }
    alloc_console();

    let mut siv = cursive::default();
    siv.set_theme(Theme::terminal_default());
    if use_preferred {
        show_configure_prefered(&mut siv);
    } else {
        show_main_menu(&mut siv);
    }
    siv.run();

    Ok(())
}
