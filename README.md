# `tree_sitter`

Elixir bindings to [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
via a Rust NIF. Currently only has support for parsing web technologies: HTML,
CSS, and JS.

This library is a work in progress and not meant for general use.

## Installation

```elixir
def deps do
  [
    {:tree_sitter, github: "linearforest/elixir-tree-sitter"}
  ]
end
```
