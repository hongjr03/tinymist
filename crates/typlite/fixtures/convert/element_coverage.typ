#set heading(numbering: "1.")

= Coverage<title>

- List item with #emph[emphasis]
- List item with #link("https://example.com")[a link]
  2. starts with a number
  + Enum item with #strong[strong text]
  + Enum item with #raw("inline code")
    - Nested list item with #underline[underlined text]
    + Enum item
- back to the first level

@title[Heading with #emph[emphasis] and #link("https://example.com")[a link]]

#quote(block: true)[A block quote with #underline[underlined text].]

#figure(
  rect(width: 16pt, height: 8pt),
  caption: [A rectangle],
)

#table(
  columns: 2,
  align: (left, right),
  table.header([A], [B]),
  [#emph[C]], table.cell(align: center)[#strong[D]],
  table.cell(colspan: 2)[E],
  table.footer([F], [G]),
)

#grid(
  columns: (1fr, 1fr),
  align: horizon,
  [G1], grid.cell(align: right)[G2],
)

#align(center)[Centered]
#box[Boxed]
#pad(x: 1pt, y: 1pt)[Padded]
#move(dx: 1pt, dy: 1pt)[Moved]
#rotate(10deg)[Rotated]
#scale(x: 80%, y: 80%)[Scaled]
#skew(ax: 10deg, ay: 0deg)[Skewed]

#highlight[Highlighted]
#smallcaps[Small caps]
#strike[Struck]
#sub[Subscript]
#super[Superscript]

$ x_1^2 + y = sqrt(4) $

```rust
fn main() {}
```
