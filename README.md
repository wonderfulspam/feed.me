# Feed.me - A curated personal homepage

RSS readers are great. What if you had one at your fingertips
available from everywhere with no software needed other than
the ubiquitous web browser?

The project consists of two parts: `spacefeeder`, a small Rust
tool which takes a list of RSS/Atom feeds as input and produces
JSON output, and a small website implementation using Zola
which uses said output to generate a static website.

All functionality is captured in the provided `justfile`.

## Small print

This project is heavily inspired by https://www.cvennevik.no/reader/
as [detailed on Mastodon](https://hachyderm.io/@cvennevik/113095066086505914).

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in in this project by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
