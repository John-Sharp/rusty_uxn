[Uxn](https://wiki.xxiivv.com/site/uxn.html) stack machine implemented in Rust. Includes:

* an assembler from the [Tal](https://wiki.xxiivv.com/site/uxntal.html) assembly language to Uxn binary program files, [uxnasmlib], invoked from the uxnasm binary crate
* a command line based machine based around Uxn, [emulators::uxnclilib], invoked from the uxncli binary crate
* a graphical machine based around Uxn (known as [Varvara](https://wiki.xxiivv.com/site/varvara.html)), [emulators::uxnemulib], invoked from the uxnemu binary crate
* utility for turning png images into Varvara compatible sequences of bytes, [utils::spritemake], invoked from the spritemake crate

# installation

```bash
cargo install --git file:/// --all-features
```

# uxnasm

The uxnasm binary is the assembler for building [Tal](https://wiki.xxiivv.com/site/uxntal.html) assembly files into uxn machine code.

## Usage

```bash
USAGE:
    uxnasm <SRC_PATH> <DST_PATH>

ARGS:
    <SRC_PATH>    The path to the assembly file
    <DST_PATH>    The path to the output rom

OPTIONS:
    -h, --help    Print help information
```

## Example

To assemble the example program located at `example_assets/cli/name_echo.tal`:

```bash
uxnasm example_assets/cli/name_echo.tal name_echo.rom
```

# uxncli

The uxncli is a command line only virtual machine built around the Uxn stack
machine. It has implementations of Varvara devices for console input/output,
file system manipulation, and date-time retrieval.

## Usage

```bash
USAGE:
    uxncli <ROM> [INPUT]...

ARGS:
    <ROM>         Rom to run
    <INPUT>...    Initial console input for uxn virtual machine

OPTIONS:
    -h, --help    Print help information
```

## Example

To assemble and then run the name echo example:

```bash
uxnasm example_assets/cli/name_echo.tal name_echo.rom && \
uxncli name_echo.rom
```

Initial console input can also be provided on the command line, with each
space separated string having a newline added at the end and being passed
to the program:

```bash
uxncli name_echo.rom you everyone
```

# uxnemu

The uxnemu is a graphical virtual machine built around the Uxn stack machine.
It has implementations of Varvara devices for console input/output, file
system manipulation, date-time retrieval, controller input, mouse input, and
writing to the screen.

## Usage

```bash
USAGE:
    uxnemu <ROM> [INPUT]...

ARGS:
    <ROM>         Rom to run
    <INPUT>...    Initial console input for uxn virtual machine

OPTIONS:
    -h, --help    Print help information
```

## Example

To assemble and then run an example allowing you to place rabbits with a
mouse:

```bash
uxnasm example_assets/emu/rabbit_test.tal rabbit_test.rom && \
uxnemu rabbit_test.rom
```
