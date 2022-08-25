[Uxn](https://wiki.xxiivv.com/site/uxn.html) stack machine implemented in Rust. Includes:

* an assembler from the [Tal](https://wiki.xxiivv.com/site/uxntal.html) assembly language to Uxn binary program files, [uxnasmlib], invoked from the uxnasm binary crate
* a command line based machine based around Uxn, [emulators::uxnclilib], invoked from the uxncli binary crate
* a graphical machine based around Uxn (known as [Varvara](https://wiki.xxiivv.com/site/varvara.html)), [emulators::uxnemulib], invoked from the uxnemu binary crate
* utility for turning png images into Varvara compatible sequences of bytes, [utils::spritemake], invoked from the spritemake crate
