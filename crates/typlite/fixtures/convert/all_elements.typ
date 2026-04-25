#set document(title: [Element Sweep], author: "Typlite")
#set page(width: 160pt, height: auto, margin: 8pt)
#set heading(numbering: "1.")
= Element Sweep<sweep>

#metadata((fixture: "all-elements"))

#block[
  Block body with #h(1em) horizontal spacing and a #linebreak() line break.
]

#columns(2, gutter: 4pt)[
  Column one
  #colbreak()
  Column two
]

#stack(dir: ttb, spacing: 2pt, [Stack A], [Stack B])

/ Term A: Description A
/ Term B: Description B

#outline(title: [Outline], target: heading)

#pagebreak(weak: true)
#v(2pt)

#place(top + right, dx: 1pt, dy: 1pt)[Placed]
#repeat([.], gap: 1pt)
#hide[Hidden]

#box[Box]
#circle(radius: 3pt)[Circle]
#ellipse(width: 8pt, height: 4pt)[Ellipse]
#rect(width: 8pt, height: 4pt)[Rect]
#square(size: 5pt)[Square]
#line(length: 8pt)
#polygon((0pt, 0pt), (4pt, 0pt), (2pt, 4pt))
#curve(
  curve.move((0pt, 0pt)),
  curve.line((4pt, 0pt)),
  curve.quad((6pt, 2pt), (8pt, 0pt)),
  curve.cubic((10pt, 2pt), (12pt, -2pt), (14pt, 0pt)),
  curve.close(),
)

#overline[Overline]
#underline[Underline]
#highlight[Highlight]
#smallcaps[Smallcaps]
#smartquote(double: true)

#footnote[Footnote body]


#ref(<sweep>)
// #cite(<missing-key>)

#pdf.artifact(kind: "page")[PDF artifact]

#table(
  columns: 2,
  table.header([H1], [H2]),
  table.hline(),
  table.cell(colspan: 2)[Merged],
  table.vline(x: 1),
  table.footer([F1], [F2]),
)

#grid(
  columns: 2,
  grid.header([GH1], [GH2]),
  grid.hline(),
  grid.cell(colspan: 2)[Grid merged],
  grid.vline(x: 1),
  grid.footer([GF1], [GF2]),
)


$ accent(x, hat) + attach(x, t: 1, b: 2) + binom(1, 2) + cancel(x) $
$ cases(1, 2) + class("normal", x) + frac(1, 2) + limits(sum, inline: #false) $
$ lr((x)) + mat(1, 2; 3, 4) + mid(|) + op("lim", limits: #true) $
$ overbrace(x, 1) + overbracket(x, 1) + overline(x) + overparen(x, 1) + overshell(x, 1) $
$ primes(#3) + root(3, x) + scripts(x) + stretch(->) $
$ underbrace(x, 1) + underbracket(x, 1) + underline(x) + underparen(x, 1) + undershell(x, 1) + vec(1, 2) $
$ vec(1, 2, delim: "[") $
$ sum_(k=0)^n k & = 1 + 2 + ... + n \
    & = (n(n+1)) / 2 $
