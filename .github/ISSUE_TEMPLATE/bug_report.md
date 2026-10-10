---
name: Bug report
about: Something in the flock is wrong
title: ""
labels: bug
assignees: ""
---

**What happened**

<!-- A clear description of the problem. -->

**What you expected**

<!-- What should have happened instead. -->

**How to reproduce it**

<!--
The exact flags, seed and terminal size. If it is visual, a screenshot or a
saved frame helps. The headless tools are useful for motion or shape problems:

  murmuration --analyze 30 --seed 42 --falcon
  murmuration --turns --seed 42
-->

**Command line**

<!-- Paste the full command, including --seed if you set one. -->

**Environment**

- murmuration version (`murmuration --version`):
- OS and version:
- Terminal (and version):
- Multiplexer, if any (tmux, screen):
- `TERM` and `COLORTERM`, if the problem is about colour:
- Font, if the problem is about braille glyphs or alignment:

**Saved frame (optional, very helpful)**

Press `s` while the problem is on screen to write an ANSI frame
(`murmuration-frame-*.ansi`) and attach it here.
