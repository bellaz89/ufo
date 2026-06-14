#[cfg(feature = "opencl")]
fn main() -> ufo::Result<()> {
    for device in ufo::opencl::list_devices()? {
        println!("{}: {}", device.index, device.name);
    }
    Ok(())
}

#[cfg(not(feature = "opencl"))]
fn main() {
    eprintln!("rebuild with the `opencl` feature to list devices");
}
