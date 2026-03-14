# zztff: A Rust library for ZZT's file formats
This crate is meant to be a flexible, somewhat low-level library for reading and writing ZZT 3.2's binary file formats.

The code was originally based on the binary parser I wrote for [Marzipan](https://github.com/cmounce/marzipan), a WIP macro language for generating ZZT files.
I extracted it from Marzipan so I could use it in a different ZZT utility, then extracted it again into this crate for easier reuse.
So over time, the code has received a fair amount of real-world testing.
But the crate *itself* is brand new, so it might have some rough edges; if you use it, let me know if you experience any issues!
