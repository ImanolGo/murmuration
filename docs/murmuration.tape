# Record the README GIF. Requires vhs (https://github.com/charmbracelet/vhs).
#
#   cargo build --release
#   vhs docs/murmuration.tape
#
Output docs/murmuration.gif

Set FontFamily "DejaVu Sans Mono"
Set FontSize 15
Set Width 960
Set Height 560
Set Padding 16
Set Framerate 12
Set CursorBlink false

Hide
Type "./target/release/murmuration --no-arc --falcon --seed 7 --birds 2000"
Enter
Sleep 3s
Show
Sleep 42s
Type "q"
Sleep 1s
