// path: /sample.svg
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <rect width="16" height="16" fill="red"/>
</svg>
-----
// path: /main.typ
#image(read("sample.svg", encoding: none), alt: "Read image")
