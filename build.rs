extern crate winresource;

fn main() -> std::io::Result<()> {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        winresource::WindowsResource::new()
            .set_icon("icon.ico")
            .set_manifest_file("manifest.xml")
            .compile()?;
    }

    Ok(())
}
