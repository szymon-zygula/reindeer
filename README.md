# Reindeer
<img
    src="https://raw.githubusercontent.com/szymon-zygula/reindeer/master/render-video.gif"
    alt="Reindeer gif"
    width="200"
/>

Reindeer is a simple 3D rendering software displaying everything directly in the terminal window
by using individual characters as pixels.
To function properly, a terminal emulator supporting True Color is required.
No external libraries (except for `libc`) are used in this project.
Rendered image is automatically resized to terminal size.
This means, that the smaller the used font is, the higher the resolution is going to be.

## Compilation
To compile Reindeer you only need `cargo`.
Compiling in `release` instead of `debug` causes massive performance improvement.
```
cargo build --release
cargo run --release
```

## Usage
Just sit back and enjoy the flying head rendered directly in your terminal.
