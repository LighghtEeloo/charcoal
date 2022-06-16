# Charcoal

An alternative to wudao-dict. With colorized output and optional speech.

## Dependency

```sh
# google_speech python library for online speech material
pip3 install google_speech

# install sox, with mp3 support, for speech
# e.g. Arch
sudo pacman -S sox libmad libid3tag twolame
```

## Usage

```sh
charcoal <word_to_query>
```

To see more options, 

```sh
charcoal --help
```

Tip: aliasing `charcoal` to `chr` or anything shorter is recommended.
