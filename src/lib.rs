use std::{
    collections::{HashMap, HashSet},
    ffi::{c_void, OsString},
    os::windows::ffi::OsStringExt,
};

use windows::{
    core::*,
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{BOOLEAN, HANDLE, HWND},
        Storage::FileSystem::{FILE_ATTRIBUTE_READONLY, FILE_SHARE_READ, OPEN_EXISTING},
        UI::{
            Input::{
                GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST,
                RAWINPUTDEVICE_FLAGS, RIDEV_INPUTSINK, RIDI_DEVICENAME,
            },
            WindowsAndMessaging::WNDCLASSEXW,
        },
    },
};
pub struct Devices {
    devices: HashSet<*mut c_void>,
    // thread_handle: Option<_>,
}

impl Devices {
    pub fn new() -> Self {
        Self {
            devices: HashSet::new(),
            //thread_handle: None,
        }
    }

    /// This starts a thread polling for new events coming from the added devices.
    /// On Windows, some parent window is required for this, and a handle to such a window can be provided via the hwnd argument.
    /// Otherwise, this will start a hidden window.
    pub fn start_listening(hwnd: Option<HWND>) {
        // a set of devices we want to listen to
        /* let device_set: HashSet<*mut c_void> = HashSet::new();

        let hwnd = match hwnd {
            Some(hwnd) => hwnd,
            None => {
                let hinstance = unsafe { GetModuleHandleW(PWSTR::null()).unwrap() };

                let classname_str = format!("RawInput Window");
                let mut classname = OsStr::new(&classname_str)
                    .encode_wide()
                    .chain(Some(0).into_iter())
                    .collect::<Vec<_>>();
                let classname = PCWSTR::from_raw(&mut classname[0]);

                let wcex = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: hinstance.into(),
                    lpfnWndProc: Some(DefWindowProcWSystem),
                }
            }
        };*/
    }

    pub fn add_device(&mut self, device: impl Device) {
        self.devices.insert(device.get_handle().0);
    }
}

pub trait Device {
    const DW_TYPE_MASK: u32;
    const USAGE_PAGE: u16;
    const USAGE_ID: u16;
    const DW_FLAG: RAWINPUTDEVICE_FLAGS;
    fn get_handle(&self) -> HANDLE;

    fn new(product_name: String, handle: HANDLE) -> Self;
}
// pub struct Mouse (+ Device)
// name: String
// handle: HANDLE
// queue: Option<Receiver> (ringbuffer perchance??)
//
// trait Device {
//      get_handle() -> HANDLE
//
//      get_queue_mut -> &mut Queue
//
//
// }
//
// pub struct Devices {
//  devices: Vec<HANDLE> // this needs to be pushable
// }
//
// pub struct Packet {
//      // granularity can only really be 1kHz since that's the usual max polling rate
//      timestamp: Instant,
//      data: Data,
// }
// impl Devices {
// Would be good if this is singleton probably
// fn register() -> ThreadHandle?? {
//        txers = HashMap::new()
//        for device in self.devices {
//          let rx, tx = Queue::new()
//          device.get_queue_mut() = rx
//          txers.insert(device.handle, tx)
//        }
//        ReceiverThread (||{
//              let _, tx = txers.get(receiving_handle)
//              tx.enqueue(packet)
//        })
// }
// }
#[derive(Debug)]
pub struct Mouse {
    pub product_name: String,
    pub handle: HANDLE,
}

impl Device for Mouse {
    // MS provided
    const DW_TYPE_MASK: u32 = 0;
    const USAGE_ID: u16 = HID_USAGE_GENERIC_MOUSE;
    const USAGE_PAGE: u16 = HID_USAGE_PAGE_GENERIC;
    const DW_FLAG: RAWINPUTDEVICE_FLAGS = RIDEV_INPUTSINK;

    fn get_handle(&self) -> HANDLE {
        todo!()
    }

    fn new(product_name: String, handle: HANDLE) -> Self {
        Mouse {
            product_name,
            handle,
        }
    }
}

pub struct Keyboard {
    pub product_name: String,
    pub handle: HANDLE,
}

impl Device for Keyboard {
    // MS provided
    const DW_TYPE_MASK: u32 = 1;
    const USAGE_ID: u16 = HID_USAGE_GENERIC_KEYBOARD;
    const USAGE_PAGE: u16 = HID_USAGE_PAGE_GENERIC;
    const DW_FLAG: RAWINPUTDEVICE_FLAGS = RIDEV_INPUTSINK;

    fn get_handle(&self) -> HANDLE {
        todo!()
    }

    fn new(product_name: String, handle: HANDLE) -> Self {
        Keyboard {
            product_name,
            handle,
        }
    }
}

pub fn get_devices<T>() -> Vec<T>
where
    T: Device,
{
    let type_mask = T::DW_TYPE_MASK;

    let mut devices_vec = vec![];
    let mut num_devices = 0;
    let device_list_size = std::mem::size_of::<RAWINPUTDEVICELIST>() as u32;

    // poll the number of devices
    // SAFETY: We are not providing a buffer, just polling the required size of the future buffer
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
    // SAFETY: Required buffer size has been polled, could this write out of bounds
    // if the amount of connected devices changes?
    unsafe {
        result = GetRawInputDeviceList(Some(&mut buffer[0]), &mut num_devices, device_list_size);
    };
    if result == u32::MAX {
        panic!("Failed to Get Raw Device List!");
    }
    // 0 = mouse, 1 = keyboard, 2 = other HID
    // get only keyboards
    let devices = buffer.iter().filter(|device| device.dwType.0 == type_mask);

    for device in devices {
        // get size of device path string

        let mut size: u32 = 0;
        // SAFETY: We are first polling the required buffer size
        let result =
            unsafe { GetRawInputDeviceInfoW(device.hDevice, RIDI_DEVICENAME, None, &mut size) };
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
        // SAFETY: Buffer has been allocated accordingly
        unsafe {
            GetRawInputDeviceInfoW(
                device.hDevice,
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
                // product string buffer must be allocated beforehand
                const SIZE: usize = 1024;
                let mut buffer: [u16; SIZE] = [0u16; SIZE];
                // get product string (typically a name)
                // SAFETY: Buffer size is handled on the OS side
                // if the string does not fit, this will fail.
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
                devices_vec.push(T::new(string.to_string(), device.hDevice));
            }
            Err(_) => {}
        }
    }
    devices_vec
}

// #[allow(non_snake_case)]
// // spicy...
// unsafe extern "system" fn DefWindowProcWSystem<P0, P1, P2>(
//     hwnd: P0,
//     msg: u32,
//     wparam: P1,
//     lparam: P2,
// ) -> Foundation::LRESULT
// where
//     P0: windows_core::Param<HWND>,
//     P1: windows_core::Param<Foundation::WPARAM>,
//     P2: windows_core::Param<Foundation::LPARAM>,
// {
//     DefWindowProcW(hwnd, msg, wparam, lparam)
// }
