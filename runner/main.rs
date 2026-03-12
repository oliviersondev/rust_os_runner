use std::{env, io, process};
use std::process::Command;
use dotenvy::dotenv;

fn main() {
    let iso = env::var("ISO").unwrap();
    println!("ISO path: {iso:?}");

    run_qemu(iso).expect("TODO: panic message");
}

fn run_qemu(output_iso: String) -> io::Result<()> {
    dotenv().ok();

    let ovmf = env::var("OVMF_PATH").unwrap();

    // Qemu runs our OS in a virtual
    let mut qemu = Command::new("qemu-system-x86_64");

    // Specify the path to the ISO
    qemu.arg("-cdrom");
    qemu.arg(output_iso.to_string());
    // For UEFI on qemu, the path to OVMF.fd is needed
    qemu.arg("-bios").arg(ovmf);

    // Pass any args to qemu
    env::args().skip(1).for_each(|arg| {
        qemu.arg(arg);
    });
    let exit_status = qemu.status()?;
    process::exit(exit_status.code().unwrap_or(1));
}
