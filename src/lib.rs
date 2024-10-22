use core::str;
use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use windows::{
    core::*,
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{BOOLEAN, HANDLE},
        Storage::FileSystem::{FILE_ATTRIBUTE_READONLY, FILE_SHARE_READ, OPEN_EXISTING},
        UI::Input::{
            GetRawInputDeviceInfoA, GetRawInputDeviceInfoW, GetRawInputDeviceList,
            RAWINPUTDEVICELIST, RIDI_DEVICEINFO, RIDI_DEVICENAME, RID_DEVICE_INFO_MOUSE,
        },
    },
};
#[derive(Debug)]
pub struct Mouse {
    pub product_name: String,
    pub handle: HANDLE,
}

pub fn get_mice() -> Vec<Mouse> {
    let mut mice_vec = vec![];
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
    for _ in 0..num_devices {
        buffer.push(RAWINPUTDEVICELIST::default());
    }

    // get devices
    unsafe {
        result = GetRawInputDeviceList(Some(&mut buffer[0]), &mut num_devices, device_list_size);
    };
    if result == u32::MAX {
        panic!("Failed to Get Raw Device List!");
    }
    // 0 = mouse, 1 = keyboard, 2 = other HID
    // get only mice
    let mice = buffer.iter().filter(|device| device.dwType.0 == 0);

    for mouse in mice {
        // get size of device path string

        let mut size: u32 = 0;
        let result =
            unsafe { GetRawInputDeviceInfoW(mouse.hDevice, RIDI_DEVICENAME, None, &mut size) };
        // if failed to get device info
        if result == u32::MAX {
            continue;
        }
        // allocate buffer for path string
        let mut path_buffer = vec![];
        for _ in 0..size {
            path_buffer.push(0u16);
        }
        // get device path string
        unsafe {
            GetRawInputDeviceInfoW(
                mouse.hDevice,
                RIDI_DEVICENAME,
                Some(std::mem::transmute(path_buffer.as_mut_ptr())),
                &mut size,
            )
        };

        // cast the path string to a windows string
        let pathstr = PWSTR::from_raw(&mut path_buffer[0]);

        // create a file handle on the raw input device
        let handle = unsafe {
            windows::Win32::Storage::FileSystem::CreateFileW(
                pathstr,
                0,
                FILE_SHARE_READ,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_READONLY,
                None,
            )
        };
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
                let result = unsafe {
                    HidD_GetProductString(
                        handle,
                        std::mem::transmute(buffer.as_mut_ptr()),
                        SIZE as u32,
                    )
                };
                if result == BOOLEAN(0) {
                    continue;
                }
                // OsString :D
                let string = OsString::from_wide(&buffer).into_string().unwrap();
                let string = string.trim_end_matches("\0");
                mice_vec.push(Mouse {
                    product_name: string.to_string(),
                    handle: mouse.hDevice,
                });
            }
            Err(_) => {}
        }
    }
    mice_vec
}

pub struct Keyboard {}

pub fn get_keyboards() -> Vec<Keyboard> {
    todo!()
}
