complete -c murmuration -l analyze -d 'Run headless for this many simulated seconds, printing flock statistics once per second, then exit' -r
complete -c murmuration -l zigzag-k -d 'Copying threshold k for the zigzag wave (1–7)' -r
complete -c murmuration -l turning -d 'Turning model: base (banked) or inertial (review C7)' -r -f -a "base\t'The bank tracks `atan(a_lat/g)` with a first-order lag'
inertial\t'The bank *rate* has momentum, after the inertial spin model (Cavagna et al. 2015): once a bird starts to roll it keeps rolling (review C7)'"
complete -c murmuration -l colors -d 'Colour handling' -r -f -a "auto\t'Detect from the environment'
truecolor\t'24-bit colour'
ansi256\t'Nearest of the 256 xterm colours'
none\t'Plain braille, no colour'"
complete -c murmuration -l arc-minutes -d 'Length of one evening, in minutes' -r
complete -c murmuration -l birds -d 'Number of birds [default: from the terminal size, 4000–8000]' -r
complete -c murmuration -l seed -d 'Random seed for the evening' -r
complete -c murmuration -l turn-side -d 'Forced-turn experiment: birds disturbed' -r
complete -c murmuration -l turn-angle -d 'Forced-turn experiment: imposed heading change, in radians' -r
complete -c murmuration -l turn-window -d 'Forced-turn experiment: seconds to simulate after the disturbance' -r
complete -c murmuration -l turn-min -d 'Forced-turn experiment: minimum local heading change to include a bird, in radians' -r
complete -c murmuration -l turns -d 'Run the forced-turn experiment headless, report the fitted wave speed and R², then exit'
complete -c murmuration -l agitation -d 'Run the agitation-wave experiment (falcon attack) headless and exit'
complete -c murmuration -l probe -d 'Run the part-D probe (2000 birds, seed 0, four minutes) and print the metrics that catch the "looks wrong" failures, then exit'
complete -c murmuration -l falcon -d 'A falcon hunts from the start'
complete -c murmuration -l screensaver -d 'Exit on any key press or mouse movement'
complete -c murmuration -l no-mouse -d 'Do not capture the mouse (no mouse falcon, no exit-on-move)'
complete -c murmuration -l no-arc -d 'Hold the display phase (no evening arc)'
complete -c murmuration -s h -l help -d 'Print help (see more with \'--help\')'
complete -c murmuration -s V -l version -d 'Print version'
