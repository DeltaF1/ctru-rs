use ctru::prelude::*;
use ctru::services::cfgu::Cfgu;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let cfgu = Cfgu::new().expect("Couldn't obtain CFGU controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("\x1b[0;0HRegion: {:?}", cfgu.region().unwrap());
    println!("\x1b[10;0HLanguage: {:?}", cfgu.language().unwrap());
    println!("\x1b[20;0HModel: {:?}", cfgu.model().unwrap());

    // Main loop
    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}
