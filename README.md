# sov

sov is a text-editor agnostic tool to help you manage your personal knowledge notes.

Why all-in on a specific editor? The philosophy behind sov is to provide a set of features to manage your notes and then letting you pick your editor of choice for writing/editing/viewing your notes: NeoVim, VSCode, Zed, ...

> **Disclaimer**: sov was primarily designed for my personal usage. I want to keep this tool as simple and as straight to the point as possible with no extra-bloat features. If it suits you as well that's great, otherwise you can check other alternatives listed below.

## Plaintext format requirements

Your plaintext notes have to respect a format for sov to work efficiently.
The idea is to make these requirements as less constraining as possible to let you build your customized system that suits you.

The requirements are:
- Wiki-links format for linking notes (e.g. `[[MyNote]]`)
- YAML metadata is located at the top of the file enclosed by three dashes (`---`)
- Headings start with one or multiple `#` at the start of a line (e.g. `# MyHeader`)
- Tags start with a `#` (e.g. `#mytag`)

This format is fully compatible with Obsidian.

## Features

sov features are provided through a CLI and a Language Server:
- Resolve note link
- List dead links
- List tags
- List orphan notes
- Create/Open daily note

## Alternatives
- https://github.com/zk-org/zk
- https://github.com/Feel-ix-343/markdown-oxide
- https://github.com/xylous/settle
