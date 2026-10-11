# Record the README GIF. Requires vhs (https://github.com/charmbracelet/vhs).
#
#   cargo build --release
#   vhs docs/murmuration.tape
#
# If browser capture stalls, use `python3 docs/record.py` (POSIX, Pillow,
# ffmpeg), which also uses gifski when available to keep the GIF small.
#
Output docs/murmuration.gif

Set FontFamily "DejaVu Sans Mono"
Set FontSize 15
Set Width 960
Set Height 560
Set Padding 16
Set Framerate 25
Set CursorBlink false

Hide
Type "./target/release/murmuration --no-arc --falcon --seed 42 --birds 4000 --colors truecolor --no-mouse"
Enter
Sleep 3s
Show
Sleep 20s
Hide
Type "q"
Sleep 1s
