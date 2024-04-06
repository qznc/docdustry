# DocDustry

Document generation industrialized for software factories.

Ok, seriously.
This is an experiment to build something similar to
[Sphinx-Needs](https://www.sphinx-needs.com/) but fast.
Linking back and forth between things is essential.

Here you will *not* write long Markdown documents.
Instead, you write many short snippets and compose them to long documents.

## Building & Running

This is a normal Rust application, so execute:

    cargo build

Run it with a command argument like `gen`:

    cargo run -- gen

Configuration is done with an ini file,
like `example_docdustry.ini`.
By default, it collects all `.md` files here
and generates HTML in `out/`.
