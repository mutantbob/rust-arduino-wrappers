This is a very experimental project to wrap C/C++ libraries for the
Arduino in Rust bindings.  In most cases you are better off using a
native rust crate for your peripheral, but many peripherals do not yet
have Rust crates (like the W5100 ethernet shield and the Adafruit
NeoPixels).

As of 2022-Feb it has (very incomplete) wrappers for
* the Ethernet library from https://github.com/arduino-libraries/Ethernet.git
* the Adafruit_NeoPixel library from https://github.com/adafruit/Adafruit_NeoPixel.git

Do not be surprised if you have to use `unsafe` methods because the safe wrappers have not yet been written.
If you are feeling ambitious submit a pull request with your upgrades.

Have a look at `ethernet-examples/src` for some examples that are basically rewrites of the examples from the
original C++ Ethernet/ library.

This git repository uses submodules to pull in the source for the
C/C++ libraries, so you should probably clone it using
```
git clone --recurse-submodules https://github.com/mutantbob/rust-arduino-wrappers.git
```

If you forgot that flag while cloning you can fix things with
```
git submodule update --init --recursive
```
You might want to read https://git-scm.com/book/en/v2/Git-Tools-Submodules some time to become more familiar with
submodules.