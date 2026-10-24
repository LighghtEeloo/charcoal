# Charcoal

An alternative to wudao-dict. With colorized output and optional speech.

## Installation

Arch (-ish):

```sh
paru -S charcoal-dict-bin
```

Brew:

```sh
brew install lighghteeloo/charcoal/charcoal-dict
```

Build from source:

```sh
cargo install charcoal-dict
```

### Small Tips Ahead

Aliasing common command to `chr`, `ww`, `wd`, or anything shorter is recommended (`cc` if you don't mind?)

```sh
alias chr="charcoal query -s"
```

No manually designated dependencies are required. See `Cargo.toml` if you are interested in the crates charcoal is using.


## Usage

### Query

```sh
charcoal query <QUERY>
```

where query can be shortened as `q`, `search`, or `s`.

With `-s` or `--speak-as true` one can force the happening of a speech.

Use `--sections` to override which content blocks are queried from the same
page and in which order they are displayed:

```sh
charcoal query concedes --sections youdao,collins
```

Use `-a`/`--all` to enable every section for one query, even when the
configuration narrows the selection (it cannot be combined with `--sections`):

```sh
charcoal query concedes -a
```

Use `--format json` for machine-readable output containing the selected
sections instead of the colorized text output:

```sh
charcoal query concedes --format json
```

### Edit

You may want to edit the configuration file in an easy way. `charcoal` gets you covered:

```sh
charcoal edit
```

And with `--reset` it will generate a brand new configuration in case anything gets wrong.

For more details on configuration file, see *Configuration* section.

### Clean

Charcoal caches up your queries, both text and audio. While it's nice for repetitive queries, it takes up some space. If you want, you can clean them with:

```sh
charcoal clean
```

For more details on cache, see *Caching Strategy* section.

### Help

To see more options, run

```sh
charcoal help
```

for help on subcommands, or

```sh
charcoal query --help
```

for each subcommand, say, `query`.


## Configuration

`sections` decides which content blocks are shown. `sections = true` enables
every section in the canonical order and is the default written for new
configurations; an explicit list selects a subset or a custom order. Each
section has its own detail switches:

```toml
sections = true

[youdao]
with_pronunciation = true
with_variants = true
with_sentence = true

[collins]
with_phonetic = true
with_word_forms = true
with_examples = true
```

For example, `sections = ["collins"]` shows only the Collins block, and
`sections = ["collins", "youdao"]` also changes the rendering order.

- `youdao`: Youdao's basic translation block (pronunciation, word forms and
  example sentences).
- `collins`: Collins English-Chinese dictionary (grouped senses and examples).

Multiple sections are introduced by a dim `▾` heading with the section name
(`▾ youdao`, `▾ collins`), while a single-section selection keeps the plain
output. If a selected section has no content for a word, a dim
`(collins: no content)` notice is printed instead of falling back to another
section.

`-c`/`--concise` is a global depth preset: it hides example sentences and
word-form lines from every section, leaving definitions and pronunciations.

Configuration files written by older versions (`main_mode` and a `[Normal]`
table) are upgraded automatically on load. The rewrite is atomic, and a failed
rewrite only logs a warning without blocking the query.

## Dev

Clone and build from source:

```sh
git clone git@github.com:LighghtEeloo/charcoal.git
cd charcoal
cargo build --all
```

Alias:

```sh
cd charcoal
cargo build --all
alias chr="RUST_LOG=info target/debug/charcoal query -s"
```

## Caching Strategy

Text and audio caches use SHA-256 filenames derived from the query, language, provider, and format version. Writes replace complete files atomically, so interrupted or concurrent queries do not expose partial entries.

Older on-disk cache entries are not reused because they lack language and format metadata. They can be removed with `charcoal cache clean`; queries repopulate the cache as needed. New exports preserve the complete cache identity.


## Pinor Arts

[Wudao-dict](https://github.com/ChestnutHeng/Wudao-dict), a command line dictionary tool written in Python, inspired me to create `charcoal`.

## License

This repo uses MIT License.

## Disclaimer and Promises

Though it's unlikely, this software may harm data. Use at your own risk.

No privacy collection will be performed.

## Credits

@TomCC7 for kindly supporting, testing and making suggestions for `charcoal`.

@BinhaoQin for providing advice (and patience!) on PKGBUILD.

@Gorgeous-Patrick for updating outdated readme.