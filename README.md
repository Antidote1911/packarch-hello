<div align="center">
  <h1>Packarch Hello</h1>
  <p><strong>Welcome screen for Packarch, written in Rust</strong></p>
</div>

Packarch Hello is the welcome screen of [Packarch](https://github.com/Antidote1911/CachyOS-Packarch),
a distribution based on CachyOS. It is a fork of
[CachyOS Hello](https://github.com/CachyOS/cachyos-welcome) 0.28.1, by Vladislav Nepogodin and the
CachyOS team, under the same license (GPL-3.0-or-later).

## Changes from CachyOS Hello

- Packarch name, logo and texts.
- French only, without language selector (and without settings file: the language was the only
  saved setting).
- Home page: only the links to the issues on GitHub and to the CachyOS packages. No read me,
  release info, get involved, wiki, forum, development, donation, Discord or Reddit buttons.
- No KDE specific code: KWin debug console, `plasma-foreground-booster`.
- The installer only checks the Internet connection: no check of the ISO version against
  cachyos.org, nor of the edition (Packarch has only the desktop edition).

The tools of CachyOS that Packarch uses are kept: tweaks (services), system fixes, DNS servers,
gaming packages, CachyOS Package Installer and Kernel Manager.

## Building

```sh
meson setup build --prefix=/usr --buildtype=release
meson compile -C build
meson install -C build
```

For development, `cargo run` launches it from the source tree.

## License

GPL-3.0-or-later, see [LICENSE](LICENSE).
