# Security policy

## What to report privately

murmuration runs locally, without network access, accounts or configuration
files. It draws to the terminal and reads keyboard and mouse events, and its
only output besides the display is a saved ANSI frame (`s`) and, with
`--analyze`/`--turns`, text on standard output.

Report privately anything that could make it act outside that drawing loop —
run code, read or write files it was not asked to, or emit terminal escape
sequences that were not intended. A crash that leaves the terminal in a broken
state (hidden cursor, un-restored alternate screen) is worth reporting too if
it is reachable from normal use.

## How to report

Use GitHub's private vulnerability reporting: open the **Security** tab of
[ImanolGo/murmuration](https://github.com/ImanolGo/murmuration/security) and
choose **Report a vulnerability**. If that is not available to you, open a
normal issue that says only that you have a security problem and asks for a
private channel; do not include the details or a reproduction in public.

Please include the version (`murmuration --version`), your platform and
terminal, and as much of a reproduction as you can share safely.

## Supported versions

Security fixes land on the latest release. There are no backports to older
versions.
