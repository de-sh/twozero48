# twozero48 🎰
A CLI implementation of 2048, in rust

## Demo
<p align="center">
    <img src="assets/demo.gif" alt="twozero48 gameplay demo" width="720">
</p>

## Installation
To play the game, first install the crate:
```sh
cargo install twozero48
```

If you want to compile from source, [ensure you have the rust tool chain installed][rustup], by going to after cloning to local and opening the directory in terminal, run
```sh
cargo run
```

## Usage

```sh
twozero48
twozero48 --board-size 5 --winning 4096
twozero48 --help

---
CONTROLS:

WASD / arrow keys: move
Q / Esc / Ctrl-C: quit
```

## Web App

The web app uses [ratzilla] to render the same Ratatui game UI in the browser!

> check it out: [shenoi.dev/twozero48](https://shenoi.dev/twozero48)

### Build

To build the web app, use trunk(requires wasm toolchain)

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk --version 0.21.14
trunk build --release
```

## Acknowledgements

1. [ratatui] - the best way to build terminal UIs. period.
2. [crossterm] - terminal handling done right.
3. [clap] - breeziest cli argument parsing.
4. [rand] - plug and play (pseudo)random number generation.
5. [criterion] - easy benchmarking!
6. [ratzilla] - eazipeazy ratatui on the web.

## License
Code in this repository is licensed under the permissive MIT license. All code contributions are by default considered to be under the same.

[rustup]: https://rustup.rs
[ratatui]: https://ratatui.rs/
[ratzilla]: https://github.com/ratatui/ratzilla
[crossterm]: https://github.com/crossterm-rs/crossterm
[clap]: https://docs.rs/clap/
[rand]: https://docs.rs/rand/
[criterion]: https://bheisler.github.io/criterion.rs/book/
