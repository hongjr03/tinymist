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
// path: /apa.csl
<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" class="in-text" version="1.0">
  <info>
    <title>Typlite Test</title>
    <id>https://example.com/typlite-test</id>
    <updated>2024-01-01T00:00:00+00:00</updated>
  </info>
  <citation>
    <layout prefix="[" suffix="]">
      <text variable="title"/>
    </layout>
  </citation>
  <bibliography>
    <layout>
      <text variable="title"/>
    </layout>
  </bibliography>
</style>
-----
// path: /main.typ
Citation: @doe2020.

#bibliography("refs.yml", title: [References], style: "apa.csl")
