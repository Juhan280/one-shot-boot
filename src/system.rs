use std::io;

use windows::Win32::Foundation::{HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
    SE_SHUTDOWN_NAME, SE_SYSTEM_ENVIRONMENT_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    TOKEN_QUERY,
};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::Shutdown::{
    EWX_FORCEIFHUNG, EWX_REBOOT, ExitWindowsEx, SHUTDOWN_REASON,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::System::WindowsProgramming::{
    GetFirmwareEnvironmentVariableExW, SetFirmwareEnvironmentVariableExW,
};
use windows::core::HSTRING;

pub fn update_priviliges() -> io::Result<()> {
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
    }?;

    let mut sys_env_luid = LUID::default();
    unsafe { LookupPrivilegeValueW(None, SE_SYSTEM_ENVIRONMENT_NAME, &mut sys_env_luid) }?;

    let mut shutdown_luid = LUID::default();
    unsafe { LookupPrivilegeValueW(None, SE_SHUTDOWN_NAME, &mut shutdown_luid) }?;

    let mut tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: shutdown_luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    unsafe { AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None) }?;
    tp.Privileges[0].Luid = sys_env_luid;
    unsafe { AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None) }?;

    Ok(())
}

pub fn get_uefi_var(name: HSTRING, guid: HSTRING) -> windows::core::Result<(u32, Vec<u8>)> {
    let mut buffer = vec![0u8; 1024];
    let mut attr = 0;
    let bytes_read = unsafe {
        GetFirmwareEnvironmentVariableExW(
            &name,
            &guid,
            Some(buffer.as_mut_ptr() as _),
            buffer.len() as u32,
            Some(&mut attr),
        )
    };

    if bytes_read > 0 {
        buffer.truncate(bytes_read as usize);
        Ok((attr, buffer))
    } else {
        Err(windows::core::Error::from_thread())
    }
}

pub fn set_uefi_var(
    name: HSTRING,
    guid: HSTRING,
    attr: Option<u32>,
    value: &[u8],
) -> windows::core::Result<()> {
    unsafe {
        SetFirmwareEnvironmentVariableExW(
            &name,
            &guid,
            Some(value.as_ptr() as *const _),
            value.len() as u32,
            attr.unwrap_or(7),
        )
    }
}

pub fn alloc_console() {
    _ = unsafe { AllocConsole() };
}

pub fn reboot() -> ! {
    unsafe { ExitWindowsEx(EWX_REBOOT | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)) }.unwrap();

    loop {
        std::thread::park();
    }
}
