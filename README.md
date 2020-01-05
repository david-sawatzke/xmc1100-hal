# xmc1100-hal

A hal for xmc1100 chips, primarily intended for the XMC2GO kit. Large portions
of this hal are based on the
[_stm32f0xx-hal_](https://github.com/stm32-rs/stm32f0xx-hal) hal.

## Flashing

The XMC2Go includes a JLink debug probe. That means can either use the segger
tools from https://segger.com or [openocd](http://openocd.org/).
Additionally also install arm-none-eabi-gdb. I am using openocd here.

First you need to start openocd

``` sh
$ openocd
```

The start gdb with the required elf file

``` sh
$ arm-none-eabi-gdb <YourElfFile>
```

And finally, connect to the gdb server and flash the chip in gdb

``` gdb
(gdb) target extended-remote localhost:3333
(gdb) load
(gdb) c
```

## FAQ
- My serial communication corrupts/returns errors/etc. Why?

  The internal clock doesn't seem to be great. On the xmc2go, this can lead to
  timing errors. You can also observe these in the arduino implementation with
  high baudrates (115200 baud), although they don't error out

- Why do interrupts not work?

  After flashing with openocd, interrupts don't work (for me). When I powercycle
  the board they work again
## Interrupts/Exceptions
[Rant time]

The xmc1100 chips have a highly unusual exception architecture, which isn't really
compatible with the normal, pretty great, cortex-m style.

(for reference, flash starts at 0x10001000, while ram start at 0x20000000)

The vector table is located in rom, which means it's not in flash and thus not
configurable (2-30 in reference manual). At startup, rom code gets executed and
then jump to the address defined at 0x10001004 with a sp in 0x10001000 (so far
so good).

But other entries are not used. The vector table in rom hard-codes the handler
addresses in ram and handlers have to be there (like in the avrs or 8051s) at
(for hardfault) 0x2000000C. The reserved area of the vector table at 0x10001010
is used for the clock config that should be configured by the rom startup code.

That means this crate has to duplicate some functionality from `cortex-m-rt`,
like the linker file stuff and also does interrupt handlers in assembly (for
guranteed size). These new interrupt handlers just emulate the style of normal
cortex-m interrupts, like Infineon does it in their
[XMC-for-Arduino](https://github.com/Infineon/XMC-for-Arduino/) project, so end
users shouldn't notice anything other than the increased interrupt latency.

## TODO
- More Peripherals
- RT is always on by default. Do we want to keep it that way?
  (It's needed so the exceptions.s has all symbols defined)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
