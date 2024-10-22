use std::{mem::MaybeUninit, time::Duration};

use windows::{
    core::*,
    Win32::UI::Input::{GetRawInputDeviceList, RAWINPUTDEVICELIST},
};

fn main() -> Result<()> {
    let mut num_devices = 0;
    let device_list_size = std::mem::size_of::<RAWINPUTDEVICELIST>() as u32;
    // poll the number of devices
    let mut result = unsafe { GetRawInputDeviceList(None, &mut num_devices, device_list_size) };
    if result == u32::MAX {
        panic!("Failed to Get Raw Device List!");
    }
    println!("num_devices {}", num_devices);

    // make space for raw input device list
    // RAWINPUTDEVICELIST is not actually a list, just an entry in the list...
    let mut buffer: Vec<RAWINPUTDEVICELIST> = vec![];
    for i in 0..num_devices {
        buffer.push(RAWINPUTDEVICELIST::default());
    }

    // get devices
    unsafe {
        result = GetRawInputDeviceList(Some(&mut buffer[0]), &mut num_devices, device_list_size);
    };
    if result == u32::MAX {
        panic!("Failed to Get Raw Device List!");
    }

    // print devices
    for idx in (0..num_devices) {
        println!("{:?}", buffer[idx as usize]);
    }

    Ok(())
}
