# calcard playground

A static, browser-only demo of [calcard](https://github.com/stalwartlabs/calcard):
bidirectional conversion between iCalendar/JSCalendar and vCard/JSContact, compiled
to WebAssembly. Everything runs client-side; nothing you paste ever leaves the page.

This is a standalone crate (its own workspace) that depends on the parent `calcard`
crate via path, so the demo always tracks the library. It is never published.

## Layout

- `src/lib.rs`: `wasm-bindgen` exports over the calcard conversion API.
- `site/`: static `index.html`, `style.css` and JS glue. The build copies the
  generated `pkg/` next to these files.
- `build.sh`: runs `wasm-pack` and assembles `dist/`.

## Build

```sh
cargo install wasm-pack   # once
./web/build.sh            # outputs web/dist/
python3 -m http.server --directory web/dist 8080
```

Then open http://localhost:8080.

## Crate features

The WASM build enables calcard's `wasm` feature (32-bit `hashify`) on top of the
default `jmap` feature, which provides the `jscalendar` and `jscontact` modules the
conversions depend on. `getrandom` is pulled in with the `js` backend so the
transitive `ahash` dependency links on `wasm32-unknown-unknown`.

## Deploy

Pushes to `main` that touch `web/`, `src/` or `Cargo.toml` trigger
`.github/workflows/pages.yml`, which builds the site and publishes it to GitHub
Pages. The `dist/` folder is a plain static bundle with no server component.
