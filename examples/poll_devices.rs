fn main() {
    let mut devices = windows_experiments::Devices::new();

    devices.add_all_devices();

    devices.start_listening(None);
}
