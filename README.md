# KORUKY

**A monochrome anime terminal interface.**

KORUKY is an open-source Rust project aiming to make anime discovery and
navigation feel natural from the terminal, with a clean hybrid TUI: minimal
navigation, useful episode details, and artwork where available.

> **Project status: early scaffold / pre-alpha.** This repository currently
> contains the workspace foundation and project documentation. It is not yet
> a functioning Crunchyroll client.

## Design direction

- **Name:** KORUKY
- **Language:** Rust
- **Interface:** Ratatui-based TUI
- **Visual style:** monochrome — black, white, and gray
- **Layout:** hybrid — minimal navigation with cover art and useful details
- **Initial platform:** Linux
- **Long-term platforms:** Arch-based distributions, Fedora, Flatpak,
  macOS, Windows, and NixOS

## Project principles

- Keep the application lightweight and responsive.
- Prefer a shared, cross-platform Rust codebase.
- Keep credentials out of KORUKY; never ask users to paste passwords or
  session cookies into the terminal.
- Use official and authorized integrations only.
- Do not scrape Crunchyroll, bypass subscriptions, or circumvent access
  controls. Until an authorized integration exists, KORUKY can only link
  users to the official website and let the browser handle sign-in and playback.
- Be transparent about features that are planned versus implemented.

## Workspace layout

```text
KORUKY/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   ├── workflows/
│   ├── CONTRIBUTING.md
│   └── SECURITY.md
├── crates/
│   ├── koruky-core/
│   ├── koruky-parser/
│   └── koruky-tui/
├── docs/
├── packaging/
│   ├── aur/
│   ├── flatpak/
│   ├── homebrew/
│   └── nix/
├── Cargo.toml
├── LICENSE
└── README.md
```

## Build

Install a stable Rust toolchain with `rustup`, then run:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo run -p koruky-tui
```

The TUI crate is currently a small placeholder so the workspace can be
extended incrementally.

## Roadmap

### Foundation
- [x] Create a Rust workspace
- [x] Separate core, command parser, and TUI crates
- [x] Add contribution, security, and issue templates
- [ ] Implement the monochrome hybrid interface
- [ ] Add command parsing and validation
- [ ] Add unit and integration tests

### Linux
- [ ] Arch package / AUR
- [ ] Flatpak
- [ ] Fedora packaging
- [ ] Test on Arch, CachyOS, Omarchy, and Fedora

### Other platforms
- [ ] macOS and Homebrew
- [ ] Windows and Winget
- [ ] Nix flake / NixOS

### Discovery and provider integration
- [ ] Search command language
- [ ] Series and episode metadata model
- [ ] Authorized provider integration, if one becomes available
- [ ] Browser hand-off to official provider pages

## Example command syntax (planned)

```text
search Frieren
watch Frieren --season 1 --episode 10 --audio ja --subs en
```

These commands are design examples, not implemented functionality yet.
Language options must reflect the actual audio and subtitle tracks offered by
the provider for the selected title and region.

## Contributing

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) before opening a pull request.
Please read [SECURITY.md](.github/SECURITY.md) before reporting a vulnerability.

## License

KORUKY is licensed under the MIT License. See [LICENSE](LICENSE).
