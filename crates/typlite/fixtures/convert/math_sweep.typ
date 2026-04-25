#set page(width: 260pt, height: auto, margin: 8pt)
= Typst Math Suite Sweep

// These examples are adapted from Typst's tests/suite/math/*.typ so the
// fixture tracks Typst's public math library surface instead of ad-hoc syntax.

== Accent
$ grave(a), acute(b), hat(f), tilde(u), macron(a), dash(a), breve(a), dot(x), dot.double(a), dot.triple(a), dot.quad(a), circle(a), acute.double(a), caron(x), arrow(Z), arrow.l(Z), arrow.l.r(Z), harpoon(a), harpoon.lt(a) $
$ accent(x + y, hat, size: #150%) $

== Attach
$ f_x + t^b + V_1^2 + attach(A, t: alpha, b: beta) $
$ attach(O, bl: 8, tl: 16, br: 2, tr: 2) $
$ attach(a, tl: u, t: t, tr: v, bl: x, b: b, br: y) $
$ limits(sum)_1^2 + scripts(sum)_1^2 $

== Fractions and roots
$ 1/2 + frac(x, y, style: "skewed") + frac(x, y, style: "horizontal") $
$ binom(n, k_1, k_2, k_3) + sqrt(x) + root(3, x) $

== Delimiters
$ lr((x + y)) + abs(x) + norm(x) + floor(x) + ceil(x) + round(x) $
$ stretch(->) + mid(|) $

== Matrix vector cases
$ mat(1, 2; 3, 4) $
$ mat(delim: "[", 1, 2; 3, 4) $
$ mat(delim: #none, 1, 2; 3, 4) $
$ vec(a, b, c) + vec(delim: "[", 1, 2) $
$ cases(1 "if" x < 0, 2 "else") $
$ cases(reverse: #true, delim: \(, z_(n_p), a^2) $

== Class operators and styles
$ class("relation", x) + op("lim", limits: #true)_(n -> oo) n $
$ upright(A) + italic(A) + bold(A) + serif(A) + sans(A) $
$ cal(A) + scr(A) + frak(A) + mono(A) + bb(A) $
$ display(sum_i x_i) + inline(sum_i x_i) + script(x) + sscript(x) $

== Under over cancel align
$ cancel(x) + cancel(x, inverted: #true) + cancel(x, cross: #true) $
$ underbrace(x + y, n) + overbrace(x + y, n) $
$ underbracket(x + y, n) + overbracket(x + y, n) $
$ underline(x + y) + overline(x + y) $
$ underparen(x + y, n) + overparen(x + y, n) $
$ undershell(x + y, n) + overshell(x + y, n) $
$ a & = b \
    && = c $
