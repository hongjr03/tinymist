// path: /refs.yml
doe2020:
  type: Article
  title: A Tiny Reference
  author: Doe, Jane
  date: 2020
  parent:
    type: Periodical
    title: Journal of Small Documents
-----
// path: /main.typ
#figure(rect(width: 10pt, height: 10pt), caption: [A figure.]) <fig-one>

Bibliography ref: @doe2020.

Figure ref: @fig-one.

#bibliography("refs.yml", title: [References], style: "apa")
