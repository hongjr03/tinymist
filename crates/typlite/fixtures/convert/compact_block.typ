#let badge(name) = box(raw(name), inset: 1pt)

#block({
  let signature = (
    raw("canvas"),
    raw("("),
    [\ ],
    h(1em),
    raw("length:"),
    [ ],
    badge("length"),
    [,],
    [\ ],
    h(1em),
    raw("x:"),
    [ ],
    badge("number"),
    [ ],
    badge("vector"),
    [,],
    [\ ],
    raw(")"),
    [ ],
    sym.arrow.r,
    [ ],
    badge("content"),
  ).join()
  signature
})
