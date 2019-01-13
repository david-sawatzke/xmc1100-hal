// Cribbed from cortex-m-rt
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cc::Build;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let link_x = include_bytes!("xmc.x.in");
    let mut f = File::create(out.join("xmc.x")).unwrap();

    f.write_all(link_x).unwrap();

    // *IMPORTANT*: The weak aliases (i.e. `PROVIDED`) must come *after* `EXTERN(__INTERRUPTS)`.
    // Otherwise the linker will ignore user defined interrupts and always populate the table
    // with the weak aliases.
    writeln!(
        f,
        r#"
/* Provides weak aliases (cf. PROVIDED) for device specific interrupt handlers */
/* This will usually be provided by a device crate generated using svd2rust (see `device.x`) */
INCLUDE device.x"#
    )
    .unwrap();

    let max_int_handlers = 32;

    // checking the size of the interrupts portion of the vector table is sub-architecture dependent
    writeln!(
        f,
        r#"
ASSERT(SIZEOF(.vector_table) <= 0x{:x}, "
There can't be more than {1} interrupt handlers. This may be a bug in
your device crate, or you may have registered more than {1} interrupt
handlers.");
"#,
        max_int_handlers * 4 + 0x40,
        max_int_handlers
    )
    .unwrap();

    Build::new().file("exceptions.s").compile("asm");

    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=exceptions.s");
    println!("cargo:rerun-if-changed=xmc.x.in");
}
