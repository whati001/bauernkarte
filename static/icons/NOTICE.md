# PWA icons

`icon.svg` and `icon-maskable.svg` are the sources; the `.png` files next
to them are generated and checked in (a manifest can't point at an SVG on
iOS, and Android wants raster icons at fixed sizes). Both draw the same
Lucide "sprout" glyph as `templates/icons/sprout.svg` and
`static/favicon.svg` — see `templates/icons/NOTICE.md` for Lucide's
license and provenance.

Regenerate after editing either SVG:

```sh
cd static/icons
rsvg-convert -w 192 -h 192 icon.svg          -o icon-192.png
rsvg-convert -w 512 -h 512 icon.svg          -o icon-512.png
rsvg-convert -w 512 -h 512 icon-maskable.svg -o icon-maskable-512.png
rsvg-convert -w 180 -h 180 icon-maskable.svg -o apple-touch-icon.png
```

(`rsvg-convert` is `librsvg`; ImageMagick's `convert` works too but
rasterises SVGs badly at these sizes unless you pass `-density`.)

The plate colour and stroke are the `--shell` and `--accent` token values
copied as literals — nothing here can read CSS custom properties. The
same two values also appear in `static/manifest.webmanifest`
(`theme_color`/`background_color`) and in `layout.html`'s `theme-color`
meta tags; changing the tokens in `static/app.css` means changing all
three by hand.
