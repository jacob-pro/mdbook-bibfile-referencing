# mdBook Bibfile Referencing

[![Build status](https://github.com/jacob-pro/mdbook-bibfile-referencing/actions/workflows/rust.yml/badge.svg)](https://github.com/jacob-pro/mdbook-bibfile-referencing/actions)
[![crates.io](https://img.shields.io/crates/v/mdbook-bibfile-referencing.svg)](https://crates.io/crates/mdbook-bibfile-referencing)

An mdBook preprocessor that uses Pandoc to add referencing to each chapter from a bibfile.

## Usage

In your `book.toml` just specify:

```
[preprocessor.bibliography]
command = "mdbook-bibfile-referencing bibliography.bib ieee.csl"
```

You must include the path to the bibliography, followed by the path to your CSL (Citation Style Language) file
which defines the style of the generated references 
(you can download pre-made ones [here](https://github.com/citation-style-language/styles)).

In each chapter of your book markdown source you can use the references in the format `[@key, PAGE_NUMBER]` -
see the [Pandoc Citeproc Docs](https://pandoc.org/demo/example19/Extension-citations.html) for the full syntax.

## Install

**Make sure you have [Pandoc Installed](https://pandoc.org/installing.html)**.

```
cargo install mdbook-bibfile-referencing
```

## Usage in CI/CD

There is a docker image: `ghcr.io/jacob-pro/mdbook-bibfile-referencing:latest` provided to make it simple and fast to 
build an mdbook in a CI system. For example in GitHub Actions you could have:

```
jobs:
  deploy:
    runs-on: ubuntu-18.04
    container:
      image: ghcr.io/jacob-pro/mdbook-bibfile-referencing:latest
    steps:
      - uses: actions/checkout@v2
      - name: Build book
        run: mdbook build
```
