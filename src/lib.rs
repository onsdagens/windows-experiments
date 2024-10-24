use std::{
    collections::HashSet,
    ffi::{c_void, OsStr, OsString},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        raw::HANDLE,
    },
};

use windows::Win32::{
    Devices::HumanInterfaceDevice::HidD_GetProductString,
    Foundation::{self, GetLastError, BOOLEAN},
};
use windows::Win32::{
    Devices::HumanInterfaceDevice::HID_USAGE_GENERIC_MOUSE,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassExW, CW_USEDEFAULT, HMENU, HWND_MESSAGE,
        WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
    },
};
use windows::Win32::{
    Devices::HumanInterfaceDevice::{HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_PAGE_GENERIC},
    UI::Input::{
        GetRawInputBuffer, RAWINPUT, RAWINPUTDEVICE_FLAGS, RAWINPUTHEADER, RIDEV_INPUTSINK,
    },
};
use windows::{
    core::PCWSTR,
    Win32::{
        Storage::FileSystem::{FILE_ATTRIBUTE_READONLY, FILE_SHARE_READ, OPEN_EXISTING},
        UI::Input::{GetRawInputDeviceInfoW, GetRawInputDeviceList, RIDI_DEVICENAME},
    },
};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RAWINPUTDEVICELIST},
            WindowsAndMessaging::WNDCLASSEXW,
        },
    },
};
pub struct Devices {
    //devices: HashSet<*mut c_void>,
    mice: Vec<Mouse>,
    keyboards: Vec<Keyboard>,
    // thread_handle: Option<_>,
}

impl Devices {
    pub fn new() -> Self {
        Self {
            mice: vec![],
            keyboards: vec![],
            //thread_handle: None,
        }
    }

    /// This starts a thread polling for new events coming from the added devices.
    /// On Windows, some parent window is required for this, and a handle to such a window can be provided via the hwnd argument.
    /// Otherwise, this will start a hidden window.
    pub fn start_listening(&self, hwnd: Option<HWND>) {
        // a set of devices we want to listen to
        let device_set: HashSet<*mut c_void> = HashSet::new();

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
                    lpszClassName: classname,
                    style: WNDCLASS_STYLES::default(),
                    ..Default::default()
                };

                let result = unsafe { RegisterClassExW(&wcex) };

                if result == 0 {
                    panic!("WindowClass Registration failed");
                }

                unsafe {
                    CreateWindowExW(
                        WINDOW_EX_STYLE::default(),
                        classname,
                        classname,
                        WINDOW_STYLE::default(),
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        HWND_MESSAGE,
                        HMENU::default(),
                        hinstance,
                        None,
                    )
                }
                .expect("Window creation failed")
            }
        };
        let mut rawinputdevices = vec![];

        let rawdevice = RAWINPUTDEVICE {
            usUsagePage: Mouse::USAGE_PAGE,
            usUsage: Mouse::USAGE_ID,
            dwFlags: Mouse::DW_FLAG,
            hwndTarget: hwnd,
        };
        rawinputdevices.push(rawdevice);

        let rawdevice = RAWINPUTDEVICE {
            usUsagePage: Keyboard::USAGE_PAGE,
            usUsage: Keyboard::USAGE_ID,
            dwFlags: Mouse::DW_FLAG,
            hwndTarget: hwnd,
        };
        rawinputdevices.push(rawdevice);

        unsafe {
            RegisterRawInputDevices(
                &rawinputdevices,
                std::mem::size_of::<RAWINPUTDEVICE>() as u32,
            )
        }
        .expect("Failed to register raw input devices");

        let mut buffer_size = 4096; // why not ...
        let mut buffer = vec![RAWINPUT::default(); buffer_size as usize];
        // this should be the callback function in new thread...
        loop {
            buffer_size = 4096;
            let n = unsafe {
                GetRawInputBuffer(
                    Some(buffer.as_mut_ptr()),
                    &mut buffer_size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                )
            };
            if n as i32 == -1 {
                panic!("failed to get input buffer: {:?}", unsafe {
                    GetLastError()
                });
            } else if n != 0 {
                for point in 0..(n as usize) {
                    unsafe {
                        println!("got some package, this is point {}", n);
                    }
                }
            }
        }
    }

    pub fn add_all_devices(&mut self) {
        //self.keyboards.extend(get_devices::<Keyboard>());
        //self.mice.extend(get_devices::<Mouse>())
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
            product_name: product_name,
            handle: handle,
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
                devices_vec.push(T::new(string.to_string(), device.hDevice.0));
            }
            Err(_) => {}
        }
    }
    devices_vec
}

#[allow(non_snake_case)]
// spicy...
unsafe extern "system" fn DefWindowProcWSystem<P0, P1, P2>(
    hwnd: P0,
    msg: u32,
    wparam: P1,
    lparam: P2,
) -> Foundation::LRESULT
where
    P0: windows_core::Param<HWND>,
    P1: windows_core::Param<Foundation::WPARAM>,
    P2: windows_core::Param<Foundation::LPARAM>,
{
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
