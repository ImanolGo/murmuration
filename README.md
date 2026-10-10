# murmuration

Starlings at dusk, in your terminal.

murmuration fills your terminal with a few thousand starlings wheeling over a reed bed at sunset. The flock folds, stretches, splits and pours back into itself, and when a falcon arrives a dark wave ripples through it. None of those shapes are animated by hand. Each bird follows a handful of rules taken from the research on real starling flocks, and everything you see comes from those rules.

> **Status:** early days. The plan lives in [PLAN.md](PLAN.md).

<!-- A GIF goes here once there is something to record. -->

## Why

A murmuration is one of the few things in nature where tens of thousands of animals make a single shape that nobody designed. There's no leader and no plan. Each starling keeps track of the six or seven birds nearest to it, and the whole sky changes shape.

I wanted to put that into the most rigid, grid-bound place I know: a terminal, a box of fixed-width characters, and see if it could still feel loose and alive there. The rules are taken from field studies and models of real flocks, not tuned by eye until it looks nice, so what you're watching is a small, honest version of the real mechanism.

It's also meant to be something worth looking at between tasks. It never loops and never repeats, it has no score and nothing to do, and it moves at the pace of the hour before dark.

And then there's the falcon. The most beautiful shapes a murmuration makes, the dark bands, the sudden bursts, the flock tearing in two and healing, are what fear looks like from far away. Every one of them is a bird trying not to be caught. That seemed worth showing honestly too.

## Where you're standing

You're on the ground, looking up and across at the flock, as you would be at a real murmuration. The simulation runs in three dimensions, and what you see is how it projects onto your sky.

That matters for how it looks. Real flocks darken and lighten not because birds bunch together, but because thousands of wings tilt at once, so that from where you're standing you suddenly see more wing and less sky. murmuration draws the flock the same way. Each bird adds darkness according to how much of its wings faces you and how far away it is. The dark bands and smoky folds come from that, not from a special effect.

## Install

You'll need a recent stable Rust toolchain.

```sh
cargo install --git https://github.com/<you>/murmuration
```

A terminal with true colour and a font with braille characters gives the best result. Most modern terminals have both.

## Usage

```sh
murmuration                 # dusk, the flock gathers
murmuration --screensaver   # any key or mouse movement exits
murmuration --birds 5000    # a bigger flock
murmuration --falcon        # a falcon hunts on its own
murmuration --seed 7        # the same evening every time
```

Keys while it's running:

| Key | Does |
| --- | --- |
| `q` / `Esc` | Quit |
| move the mouse | You're the falcon: the flock reacts to the cursor |
| `f` | Release a falcon that hunts on its own |
| `Space` | Pause |
| `+` / `-` | More or fewer birds |
| `c` | Move the camera to a new spot on the ground |
| `s` | Save the current frame as a text file |

## The science, briefly

The flock is built on the **StarDisplay** model by Hanno Hildenbrandt, Claudio Carere and Charlotte Hemelrijk, which was designed specifically for starlings and checked against field data. In it, each bird:

- **pays attention to a fixed number of neighbours, not a fixed distance.** It widens or narrows its attention until about six or seven birds are in view. Ballerini and colleagues found that real starlings behave this way by reconstructing flocks of thousands of birds in 3D from photographs over Rome.
- **keeps its distance, stays with its neighbours, and flies in roughly the same direction.** This is familiar from Reynolds' classic "boids". The difference is that birds at the edge of the flock are pulled inward more strongly, which keeps the flock's boundary crisp.
- **flies like a bird, not a dot.** Each bird has a cruising speed it returns to, and to turn it rolls (banks) into the turn the way a small plane does. Classic boids can turn instantly; birds can't, and that difference is a big part of why boids look artificial.
- **is drawn back to the roost.** A gentle pull toward the reed bed and a preferred height keeps the flock overhead and flattens it into the thin sheet real murmurations form.

Two more pieces of research shape what happens when a falcon attacks:

- **The dark waves.** Hemelrijk and colleagues showed that the dark bands running through a flock under attack are most likely birds rolling sideways and back, a quick zigzag copied from neighbour to neighbour, rather than birds packing closer together. When their simulated birds were drawn as balls instead of winged shapes, the waves vanished. murmuration uses the same copying rule.
- **The repertoire.** Storms and colleagues catalogued nearly 800 collective escape events above Rome: blackening, waves, splits, flash expansion (a sudden outward burst), cordons (two halves joined by a thin string of birds) and vacuoles (a hole opening in the flock). These aren't scripted here. They're what to look for, and a way to check the simulation is behaving like the real thing.

Some things are simplified or still open. StarDisplay makes the edge of the flock slightly less dense than the middle, while real flocks are denser at the edge. The flat, sheet-like shape comes partly from the preferred height rather than emerging on its own. And the zigzag explanation of dark waves comes from models; nobody has yet filmed individual birds doing it inside a wild flock.

## References

**Starling flocks in the field**

1. Ballerini, M., Cabibbo, N., Candelier, R., Cavagna, A., Cisbani, E., Giardina, I., Lecomte, V., Orlandi, A., Parisi, G., Procaccini, A., Viale, M. & Zdravkovic, V. (2008). Interaction ruling animal collective behavior depends on topological rather than metric distance: evidence from a field study. *PNAS* 105(4), 1232–1237. https://doi.org/10.1073/pnas.0711437105
2. Ballerini, M., Cabibbo, N., Candelier, R., Cavagna, A., Cisbani, E., Giardina, I., Orlandi, A., Parisi, G., Procaccini, A., Viale, M. & Zdravkovic, V. (2008). An empirical study of large, naturally occurring starling flocks: a benchmark in collective animal behaviour. *Animal Behaviour* 76(1), 201–215. https://arxiv.org/abs/0802.1667
3. Cavagna, A., Cimarelli, A., Giardina, I., Parisi, G., Santagati, R., Stefanini, F. & Viale, M. (2010). Scale-free correlations in starling flocks. *PNAS* 107(26), 11865–11870. https://doi.org/10.1073/pnas.1005766107
4. Attanasi, A., Cavagna, A., Del Castello, L., Giardina, I., Grigera, T. S., Jelić, A., Melillo, S., Parisi, L., Pohl, O., Shen, E. & Viale, M. (2014). Information transfer and behavioural inertia in starling flocks. *Nature Physics* 10, 691–696. https://doi.org/10.1038/nphys3035
5. Procaccini, A., Orlandi, A., Cavagna, A., Giardina, I., Zoratto, F., Santucci, D., Chiarotti, F., Hemelrijk, C. K., Alleva, E., Parisi, G. & Carere, C. (2011). Propagating waves in starling, *Sturnus vulgaris*, flocks under predation. *Animal Behaviour* 82(4), 759–765.
6. Storms, R. F., Carere, C., Zoratto, F. & Hemelrijk, C. K. (2019). Complex patterns of collective escape in starling flocks under predation. *Behavioral Ecology and Sociobiology* 73, 10. https://doi.org/10.1007/s00265-018-2609-0

**Models**

7. Reynolds, C. W. (1987). Flocks, herds and schools: a distributed behavioral model. *Computer Graphics (SIGGRAPH '87)* 21(4), 25–34. https://doi.org/10.1145/37402.37406
8. Hildenbrandt, H., Carere, C. & Hemelrijk, C. K. (2010). Self-organized aerial displays of thousands of starlings: a model. *Behavioral Ecology* 21(6), 1349–1359. https://arxiv.org/abs/0908.2677
9. Hemelrijk, C. K. & Hildenbrandt, H. (2011). Some causes of the variable shape of flocks of birds. *PLoS ONE* 6(8), e22479. https://doi.org/10.1371/journal.pone.0022479
10. Hemelrijk, C. K., van Zuidam, L. & Hildenbrandt, H. (2015). What underlies waves of agitation in starling flocks. *Behavioral Ecology and Sociobiology* 69(5), 755–764. https://doi.org/10.1007/s00265-015-1891-3
11. Costanzo, A., Hildenbrandt, H. & Hemelrijk, C. K. (2022). Causes of variation of darkness in flocks of starlings, a computational model. *Swarm Intelligence* 16, 91–105.
12. Cavagna, A., Del Castello, L., Giardina, I., Grigera, T., Jelić, A., Melillo, S., Mora, T., Parisi, L., Silvestri, E., Viale, M. & Walczak, A. M. (2015). Flocking and turning: a new model for self-organized collective motion. *Journal of Statistical Physics* 158, 601–627. https://arxiv.org/abs/1403.1202
13. Pearce, D. J. G., Miller, A. M., Rowlands, G. & Turner, M. S. (2014). Role of projection in the control of bird flocks. *PNAS* 111(29), 10422–10426. https://arxiv.org/abs/1407.2414

References 12 and 13 aren't in the main model but are available as experiments (see PLAN.md).

## License

MIT, see [LICENSE](LICENSE).
