use windows_experiments::{Keyboard, Mouse};

fn main() {
    let keyboards = windows_experiments::get_devices::<Keyboard>();
    let mice = windows_experiments::get_devices::<Mouse>();

    println!("Mice: ");
    for mouse in mice {
        println!("{}", mouse.product_name);
    }
    println!("Keyboards: ");
    for keyboard in keyboards {
        println!("{}", keyboard.product_name);
    }
}
