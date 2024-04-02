# sov

sov is a text-editor agnostic tool to help you manage your personal knowledge notes.

Why all-in on a specific editor? The philosophy behind sov is to provide a set of features to manage your notes and then letting you pick your editor of choice for writing/editing/viewing your notes: NeoVim, VSCode, Zed, ...

> **Disclaimer**: sov was primarily designed for my personal usage. I want to keep this tool as simple and as straight to the point as possible with no extra-bloat features. If it suits you as well that's great, otherwise you can check other alternatives listed below.

## Plaintext format requirements

Your plaintext notes have to respect a certain format for sov to work.
The idea is to make these requirements as less constraining as possible to let you build a customized system that suits you.

The requirements are:
- Wiki-links format for linking notes (e.g. `[[MyNote]]`)
- YAML metadata is located at the top of the file enclosed by three dashes (`---`)
- Headings start with one or more `#` at the start of a line (e.g. `## MyHeader`)
- Tags start with a `#` and contain no space (e.g. `#mytag`)

This format is fully compatible with Obsidian.

## Features

sov currently supports the following features:
- Resolve note link
- List dead links
- List tags
- List orphan notes
- Create/Open daily note

## Usage

sov features are provided through a CLI and a Language Server.

## Alternatives

- https://github.com/zk-org/zk
- https://github.com/Feel-ix-343/markdown-oxide
- https://github.com/xylous/settle

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to this project.

## License

SyncDisBoi is licensed under the GNU AGPLv3 license. Refer to [LICENSE](LICENSE.txt) for more information.

## Support

Your support helps me continue to maintain and improve this project. If you find sov useful and want to show your appreciation, consider sponsoring or donating:
- GitHub Sponsors: Preferred method. You can sponsor me on [GitHub Sponsors](https://github.com/sponsors/SilentVoid13). 
- PayPal: You can also make a donation via [PayPal](https://www.paypal.com/donate?hosted_button_id=U2SRGAFYXT32Q).

Every bit of support is greatly appreciated!

[![GitHub Sponsors](https://img.shields.io/github/sponsors/silentvoid13?label=Sponsor&logo=GitHub%20Sponsors&style=for-the-badge)](https://github.com/sponsors/silentvoid13)
[![Paypal](https://img.shields.io/badge/paypal-silentvoid13-yellow?style=social&logo=paypal)](https://www.paypal.com/donate?hosted_button_id=U2SRGAFYXT32Q)
