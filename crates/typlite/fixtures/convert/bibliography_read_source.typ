// path: /refs.bib
@article{doe2020,
  author = {Doe, Jane},
  title = {A Tiny Reference},
  journaltitle = {Journal of Small Documents},
  date = {2020},
}
-----
// path: /main.typ
Reference citation: @doe2020.

#bibliography(read("refs.bib", encoding: none), title: [References], style: "apa")
