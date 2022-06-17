# Charcoal

An alternative to wudao-dict. With colorized output and optional speech.

## Installation

Just as usual for normal developing Rust crates:

```sh
git clone git@github.com:LighghtEeloo/Charcoal.git ./charcoal
cd charcoal
cargo build --all
```

As the crate becomes stable, publishing via cargo, AUR and other major package managers will be available.

## Dependency

Almost everything is self-contained, unless you need the speech utility:

```sh
# google_speech python library for online speech material
pip3 install google_speech

# install sox, with mp3 support, for playing sound
# e.g. Arch
sudo pacman -S sox libmad libid3tag twolame
```

## Usage

```sh
charcoal <QUERY>
```

To see more options,

```sh
charcoal --help
```

Tip: aliasing `charcoal` to `chr` or anything shorter is recommended (`cc` if you don't mind?)

```sh
alias chr="charcoal -s"
```

For debug:

```sh
cd charcoal
alias chr="RUST_LOG=info target/debug/charcoal -s"
```
