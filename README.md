# sov

sov is a text-editor agnostic tool to help you manage your personal knowledge notes.

Why all-in on a specific note-taking tool? The philosophy behind sov is to provide a set of basic features to help you manage your notes either through a CLI or an LSP. You can then use your editor of choice to write/edit/view your notes: Neovim, VSCode, Zed, ...

> **Disclaimer**: sov was primarily designed for my personal usage. I want to keep this tool as simple and as straight to the point as possible with no extra-bloat features. If it suits you as well that's great, otherwise you can check other alternatives listed below.

## Plaintext format requirements

Your plaintext notes have to respect a certain format for sov to work.
The idea is to make these requirements as less constraining as possible to let you build your own customized system.

The requirements are:
- Your notes are markdown files
- Wiki-links are used to link notes (e.g. `[[MyNote]]`)
    - Linking to a header is supported (e.g. `[[MyNote#MyHeader]]`)
    - Aliases are supported (e.g. `[[MyNote|MyNoteAlias]]`)
- YAML metadata is located at the top of the file enclosed by three dashes (`---`)
    - The `aliases` key is used to set aliases for the note
    - The `tags` key is used to set tags for the note

This format is fully compatible with Obsidian.

## Features

sov currently offers the following features:
- List
    - Tags
    - Dead links: notes that are linked to, but do not exist
    - Orphan notes: notes that are not linked to any other note
- Search
    - All notes with a specific tag
- Resolve note link
- Rename note and update all backlinks
- Create/Open daily note

## Config

The sov configuration file is located at `~/.config/sov/sov.toml`:

```toml
# Root directory for your personal knowledge notes
notes_dir = "<personal_knowledge_dir>"
# Directory for scripts
scripts_dir = "<scripts_dir>"
# Directory for new daily notes
daily_notes_dir = "<daily_notes_dir>"
# Script to use for new daily note content
daily_notes_script = ""
# List of directories that will be ignored by sov
ignore_dirs = []
```

## Usage

sov features are provided through a CLI and a Language Server.

### CLI

```txt
Usage: sov [OPTIONS] <COMMAND>

Commands:
  index
  list
  resolve
  rename
  script
  search
  daily
  help     Print this message or the help of the given subcommand(s)

Options:
  -s, --silent
  -h, --help     Print help
  -V, --version  Print version
```

### LSP

- [sov.nvim](https://github.com/SilentVoid13/sov.nvim): an implementation of the LSP for Neovim

Feel free to contribute and develop the LSP implementation for your favorite editor.

## Alternatives

- https://github.com/zk-org/zk
- https://github.com/Feel-ix-343/markdown-oxide
- https://github.com/xylous/settle

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to this project.

## License

sov is licensed under the GNU AGPLv3 license. Refer to [LICENSE](LICENSE.txt) for more information.

## Support

Your support helps me continue to maintain and improve this project. If you find sov useful and want to show your appreciation, consider sponsoring or donating:
- GitHub Sponsors: Preferred method. You can sponsor me on [GitHub Sponsors](https://github.com/sponsors/SilentVoid13). 
- PayPal: You can also make a donation via [PayPal](https://www.paypal.com/donate?hosted_button_id=U2SRGAFYXT32Q).

Every bit of support is greatly appreciated!

[![GitHub Sponsors](https://img.shields.io/github/sponsors/silentvoid13?label=Sponsor&logo=GitHub%20Sponsors&style=for-the-badge)](https://github.com/sponsors/silentvoid13)
[![Paypal](https://img.shields.io/badge/paypal-silentvoid13-yellow?style=social&logo=paypal)](https://www.paypal.com/donate?hosted_button_id=U2SRGAFYXT32Q)
