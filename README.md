# rustyquarium

ASCII aquarium animation for your terminal, written in Rust with `crossterm`.

This project is a Rust port of [goquarium](https://github.com/sergev/goquarium),
which is itself a Go rewrite of the classic asciiquarium.

## Features

- Animated fish, sharks, whales, ships, monsters, and other sea creatures
- Environment elements like water lines, seaweed, and castle decoration
- Color rendering with per-character color masks
- Interactive keyboard controls
- Works in standard terminals on Linux/macOS/Windows (with terminal support)

## Requirements

- Rust 1.70+
- A terminal with color support
- Recommended terminal size: at least `40x15`

## Install

```bash
cargo install --path .
```

The binary is installed to `$HOME/.cargo/bin/rustyquarium`.
Make sure `$HOME/.cargo/bin` is in your PATH.

## CLI Options

```bash
rustyquarium --version
rustyquarium --info
rustyquarium --classic
```

## Controls

- `q` - Quit
- `p` - Pause/unpause
- `r` - Reset and respawn entities
- `i` - Toggle info overlay
- `Esc` - Close info overlay

## Development

Build (debug):

```bash
cargo build
```

Build (release):

```bash
make
```

Run:

```bash
./target/debug/rustyquarium
```

Install, uninstall:

```bash
make install
make uninstall
```

By default, `make install` installs to `$HOME/.local/usr/bin/rustyquarium`.

## Documentation

- [Visual Entity Catalog](Entities.md) — Full reference for all rendered entities,
  grouped by class with implementation details.

## Credits

- [Original Perl version `asciiquarium`](http://robobunny.com/projects/asciiquarium): Kirk Baucom
- [Python port `asciiquarium-python`](https://github.com/MKAbuMattar/asciiquarium-python): Mohammad Abu Mattar
- [Go port `goquarium`](https://github.com/sergev/goquarium): Serge Vakulenko

## License

GNU GPL v3. See [LICENSE](LICENSE).
