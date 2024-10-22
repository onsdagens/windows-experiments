use std::{mem::MaybeUninit, time::Duration};

use windows::{
    core::*,
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::GENERIC_READ,
        Storage::FileSystem::{FILE_ATTRIBUTE_READONLY, FILE_SHARE_READ, OPEN_EXISTING},
        UI::Input::{
            GetRawInputDeviceInfoA, GetRawInputDeviceInfoW, GetRawInputDeviceList,
            RAWINPUTDEVICELIST, RIDI_DEVICENAME,
        },
    },
};

fn main() -> Result<()> {
    let mut num_devices = 0;
    let device_list_size = std::mem::size_of::<RAWINPUTDEVICELIST>() as u32;
    // poll the number of devices
    let mut result = unsafe { GetRawInputDeviceList(None, &mut num_devices, device_list_size) };
    if result == u32::MAX {
        panic!("Failed to Get Raw Device List!");
    }

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
    for i in 0..num_devices {
        unsafe {
            let mut size: u32 = 0;
            // get size of device path
            let result = GetRawInputDeviceInfoW(
                buffer[i as usize].hDevice,
                RIDI_DEVICENAME,
                None,
                &mut size,
            );
            // allocate device path buffer
            let mut path_buffer = vec![];
            for i in 0..size {
                path_buffer.push(0u16);
            }
            // get device path
            GetRawInputDeviceInfoW(
                buffer[i as usize].hDevice,
                RIDI_DEVICENAME,
                Some(std::mem::transmute(path_buffer.as_mut_ptr())),
                &mut size,
            );

            // cast the path buffer to a windows string
            let pathstr = PWSTR::from_raw(&mut path_buffer[0]);

            // create a file handle on the raw input device
            let handle = windows::Win32::Storage::FileSystem::CreateFileW(
                pathstr,
                0,
                FILE_SHARE_READ,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_READONLY,
                None,
            );

            match handle {
                Ok(handle) => {
                    // if handle could be created, print device info
                    print!("Device: ");
                    for c in &path_buffer {
                        print!("{}", char::from_u32(*c as u32).unwrap());
                    }
                    print!("\n");
                    // product string buffer must be allocated beforehand
                    const SIZE: usize = 1024;
                    let mut buffer: [u16; SIZE] = [0u16; SIZE];

                    // get product string (typically a name)
                    let result = HidD_GetProductString(
                        handle,
                        std::mem::transmute(buffer.as_mut_ptr()),
                        SIZE as u32,
                    );
                    for c in buffer {
                        print!("{}", char::from_u32(c as u32).unwrap());
                    }
                    print!("\n");
                    println!("-----------------");
                }
                Err(_) => {}
            }
        }
    }
    Ok(())
}
