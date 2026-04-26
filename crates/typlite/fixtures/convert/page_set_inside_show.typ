#let styled(body) = {
  set page(header: [Ignored in typlite])
  body
}

#show: styled

= Title

Hello world.
