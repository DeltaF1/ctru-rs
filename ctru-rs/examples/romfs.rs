use ctru::prelude::*;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    cfg_if::cfg_if! {
        // Run this code if RomFS are wanted and available
        // This never fails as `ctru-rs` examples inherit all of the `ctru` features,
        // but it might if a normal user application wasn't setup correctly
        if #[cfg(all(feature = "romfs", romfs_exists))] {
            let _romfs = ctru::services::romfs::RomFS::new().unwrap();

            let f = std::fs::read_to_string("romfs:/test-file.txt").unwrap();
            println!("Contents of test-file.txt: \n{f}\n");

            let f = std::fs::read_to_string("romfs:/ファイル.txt").unwrap();
            // While RomFS supports UTF-16 file paths, `Console` doesn't...
            println!("Contents of [UTF-16 File]: \n{f}\n");
        } else {
            println!("No RomFS was found, are you sure you included it?")
        }
    }

    println!("\nPress START to exit");

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
