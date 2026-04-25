#let node(kind, attrs: (:), body) = html.elem(
  "typlite-" + kind,
  attrs: attrs,
  body,
)

#let inline(kind, attrs: (:), body) = html.elem(
  "span",
  attrs: attrs + (data-typlite: kind),
  body,
)

#let typlite(body) = {
  show heading: it => node("heading", attrs: (level: str(it.level)), it.body)
  show par: it => node("paragraph", it.body)
  show emph: it => inline("emph", it.body)
  show strong: it => inline("strong", it.body)
  show raw: it => inline("raw", attrs: (lang: if it.lang == none { "" } else { it.lang }), it.text)

  body
}
