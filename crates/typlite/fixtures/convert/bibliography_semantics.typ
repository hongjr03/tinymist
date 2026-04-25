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
Reference citation: @doe2020.

Function citation: #cite(<doe2020>).

#bibliography("refs.yml", title: [References], style: "apa")
