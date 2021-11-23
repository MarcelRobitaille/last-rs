# `last.rs`

> A Rust reimplementation of the [`util-linux`](https://github.com/util-linux/util-linux) [`last`](https://github.com/util-linux/util-linux/blob/master/login-utils/last.c) command.

**This is NOT intended as a replacement for `last.c`.**
It is mostly a library to provide `last`-like output to other Rust project.
It is also an example of how to use [utmp-rs](https://github.com/upsuper/utmp-rs).

**NOT yet feature compatible with `last.c`.**
Right now, the output is kind of like `last <username>`, and no options or flags are supported.

## Acknowledgements

Thank you to [utmp-rs](https://github.com/upsuper/utmp-rs) for doing the hard work of parsing `/var/log/wtmp` and making my life easy.
