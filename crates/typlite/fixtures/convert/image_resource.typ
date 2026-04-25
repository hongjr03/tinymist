// path: sample.svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="8" viewBox="0 0 16 8">
  <rect width="16" height="8" fill="#eeeeee" stroke="#111111"/>
</svg>

-----
// path: main.typ
= Image Resource

#image("sample.svg", alt: "Sample [box]")

#figure(
  image("sample.svg", alt: "Figure sample"),
  caption: [Image figure],
  alt: "Figure [alt]",
)
