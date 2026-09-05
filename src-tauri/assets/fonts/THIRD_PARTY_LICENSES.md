# Bundled fonts — third-party licenses

ams bundles its default font set by embedding it at compile time via
`typst-kit`'s `embedded-fonts` feature (`typst_kit::fonts::embedded()`), which
in turn embeds the same font files the Typst CLI itself ships, from the
`typst-assets` crate (pinned to `=0.15.1` alongside the rest of the `typst`
crates in `../../Cargo.toml`). No font binaries are committed to this
repository — they're compiled into the app from the `typst-assets` crate.

| Font family | Role | License |
|---|---|---|
| Libertinus Serif | default text font | SIL Open Font License 1.1 |
| New Computer Modern / New Computer Modern Math | default math font | GUST Font License 1.0 |
| DejaVu Sans Mono | default code font | Bitstream Vera Fonts Copyright (DejaVu fonts license) |

All three licenses permit redistribution (including bundling with software)
free of charge, which is what makes them suitable for a zero-network,
zero-system-fonts install. The full, authoritative license texts are
reproduced in `typst-assets`' own `NOTICE` file:
<https://github.com/typst/typst-assets/blob/main/NOTICE> (see the `v0.15.1`
tag for the exact version pinned here).
