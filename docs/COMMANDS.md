# Command language (planned)

The command language is a planned feature. The examples below describe the
intended interface and are not implemented in this scaffold.

```text
search Frieren
watch Frieren --season 1 --episode 10 --audio ja --subs en
```

Proposed language codes:
- `ja`: Japanese audio
- `en`: English audio or subtitles
- `de`: German audio or subtitles

The application must distinguish audio language from subtitle language.
Availability is title- and region-dependent and must never be assumed.
