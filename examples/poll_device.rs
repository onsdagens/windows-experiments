use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::Instant;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{self, GetLastError};
use windows::Win32::UI::Input::{GetRawInputBuffer, RAWINPUT, RAWINPUTHEADER, RAWINPUT_0};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW, CW_USEDEFAULT, HMENU, HWND_MESSAGE,
    WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES,
};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::{RegisterRawInputDevices, RAWINPUTDEVICE},
            WindowsAndMessaging::WNDCLASSEXW,
        },
    },
};
use windows_experiments::{Device, Mouse};
fn main() {
    let mice = windows_experiments::get_devices::<Mouse>();
    // typically if youre running a GUI application, you already have a window for which you can provide a HWND
    // in this example case, we create one specifically for this purpose.
    let mut set = HashMap::new();
    for mouse in mice {
        set.insert(mouse.handle.0, mouse.product_name);
        println!("mouse handle: {:?}", mouse.handle.0);
    }
    let hwnd: HWND;
    let hinstance = unsafe {
        // if input is NULL, the returned handle is to the calling process
        // if this fails, there is not much to do but panic in any case
        // SAFETY: Hope the OS does the right thing
        GetModuleHandleW(PWSTR::null()).unwrap()
    };
    let classname_str = format!("RawInput Window");
    let mut classname = OsStr::new(&classname_str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    let classname = PCWSTR::from_raw(&mut classname[0]);
    //let classname = PCWSTR::null();
    let wcex = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance.into(),
        lpfnWndProc: Some(DefWindowProcWSystem),
        lpszClassName: classname,
        style: WNDCLASS_STYLES::default(),
        // nulls all other values, this is handled on OS side.
        ..Default::default()
    };

    let result = unsafe { RegisterClassExW(&wcex) };

    if result == 0 {
        panic!("WindowClass Registration failed");
    }

    hwnd = unsafe {
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
    .expect("Window creation failed");

    let input_device = RAWINPUTDEVICE {
        usUsagePage: Mouse::USAGE_PAGE,
        usUsage: Mouse::USAGE_ID,
        dwFlags: Mouse::DW_FLAG,
        //hwndTarget: HWND::default(),
        hwndTarget: hwnd,
    };

    unsafe {
        RegisterRawInputDevices(
            &[input_device],
            std::mem::size_of::<RAWINPUTDEVICE>() as u32,
        )
    }
    .expect("Failed to register input device");

    let mut buffer_size = 0;
    let n = unsafe {
        GetRawInputBuffer(
            None,
            &mut buffer_size,
            std::mem::size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    if n as i32 == -1 {
        panic!("failed");
    }
    let mut buffer_size: u32 = 4096; // why not .......
    let mut buffer = vec![RAWINPUT::default(); buffer_size as usize];
    let start = Instant::now();
    let mut timestamp = start;
    loop {
        // docs say this is written to only if the buffer pointer is null,
        // that's not true, it gets overwritten with each call.
        let mut buffer_size: u32 = 4096;
        let n = unsafe {
            GetRawInputBuffer(
                Some(buffer.as_mut_ptr()),
                &mut buffer_size,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            )
        };
        if n != 0 {
            let now = Instant::now();
            let delta = now - timestamp;
            timestamp = now;
            if n as i32 == -1 {
                println!("failed to get input buffer: {:?}", unsafe {
                    GetLastError()
                });
            }
            for point in 0..(n as usize) {
                unsafe {
                    println!(
                        "{{{}}}:{} moved: x: {}, y: {}",
                        delta.as_micros(),
                        set.get(&buffer[point].header.hDevice.0).unwrap(),
                        buffer[point].data.mouse.lLastX,
                        buffer[point].data.mouse.lLastY
                    )
                }
            }
        }
    }
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
