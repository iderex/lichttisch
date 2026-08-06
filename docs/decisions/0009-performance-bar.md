# 0009 The performance bar the first release has to clear

Status: accepted
Date: 2026-08-06
Issue: #12

## The question

The readme says fast and fluid at a hundred thousand-plus images. Neither word
is a number, and a word cannot be missed. This record turns both into numbers
before the code that would be tuned against them exists, so the bar is a target
somebody set rather than a description of whatever was built.

Every number below is a target. None of them is a measurement, no measurement of
any of them exists at the time of writing, and the harness that would produce
one is issue #25. Where a number here turns out to be unreachable, this record
is corrected in a change that says why. A target quietly missed at release time
is the failure this record exists to prevent.

## The reference machine

One specification, named here, so that a number in an issue, a number in a
decision record and a number in a check are the same number. Issue #25's harness
prints the machine it ran on and takes this specification as the one it is
comparing against; it does not define a second one.

- eight physical cores, x86-64 with AVX2, no accelerator of any kind
- 32 GB of memory
- the catalogue, the preview store and the photographs on NVMe solid state
  storage, all on the machine, none on a network share
- one display at 2560 by 1440 running at 60 Hz
- a release build, not a debug build

No accelerator is part of the specification rather than an omission. Issue #59
owns what one buys, and every target below has to hold without one, because the
machine a photographer already owns is the machine this has to be fast on.

The corpus is the generated one from issue #4, at 100,000 files, with its seed
stated by the harness. Nothing below is measured against real photographs, and
nothing below says anything about culling quality. That is issue #62's ground
and not this record's.

## The frame budget, and why the percentile is high

A frame budget is a claim about a display refresh rate. At the 60 Hz of the
reference machine, one frame is:

    python -c "print(1000.0 / 60.0)"
    16.666666666666668

so the budget is 16.7 ms, and a target stated in milliseconds without the refresh
rate behind it is not a target anybody can check.

An interactive surface is judged by its worst frames. A median frame time says
nothing about whether scrolling felt smooth, because the frames a photographer
notices are exactly the ones the median discards. So frame timing is stated at
the 99th percentile, and one-shot operations at the 95th. Both are over a stated
sample count that the harness prints, and a target with no percentile beside it
is not one of these.

## The targets

Each one names where the measurement starts and where it stops, because two
people measuring the same word differently produce two numbers and neither is
wrong.

**Opening a catalogue.** From the process being started with a catalogue of
100,000 photographs to the first screen of the grid being drawn with real
thumbnails rather than placeholders. 2.0 s at the 95th percentile. The catalogue
is warm on disk and cold in memory, which is the ordinary case: the photographer
opened it yesterday.

**Scrolling the grid.** A continuous scroll of at least 10 s through a grid of
100,000 items, driven at a constant rate rather than by hand. 16.7 ms at the
99th percentile, and no single frame over 33.3 ms, which is one dropped frame at
the reference refresh rate. A cell whose preview is not yet decoded may be drawn
as a placeholder, and the placeholder must be replaced within 250 ms of the cell
coming to rest, at the 95th percentile. Without that second half the first half
is trivially met by drawing nothing.

**Filtering the whole library.** From the filter being committed to the first
screen of results being drawn, over all 100,000 rows, for a filter that no
prepared index answers directly. 250 ms at the 95th percentile. The result count
is part of the result: a filter that draws a screen and leaves the count spinning
has not finished.

**Comparing two frames at full resolution.** From the comparison being requested
to both frames being drawn at one screen pixel per image pixel at the centre of
each. 1.0 s at the 95th percentile for the pair, and 400 ms for the first of the
two, because the photographer starts looking at the first one while the second
arrives. A camera-embedded preview does not satisfy this target; issue #41 owns
where a preview is allowed to stand in and this is not one of those places.

**Importing a corpus.** From the import being started over a tree of 100,000
files to the last file being queryable in the catalogue. 20 minutes on the
reference machine:

    python -c "print(100000 / (20 * 60))"
    83.33333333333333

so 83 files per second sustained, stated as a median over the run rather than a
percentile because it is a throughput figure. Beside it, the number a
photographer actually meets: a single shoot of 2,000 files queryable within 30
seconds, which is the sustained rate plus room for the fixed cost of starting an
import.

That last pair is the one target on this list set from what the work has to feel
like rather than from a property of the hardware, and it is the one most likely
to be wrong. Issues #28 and #29 are where it meets reality.

**Tethering.** Record 0004 already set two figures, from the shutter release to
the frame being visible and from the shutter release to it being ranked and
comparable, both at the 95th percentile. This record adopts them unchanged and
does not copy them, because two records holding one number is how the two stop
agreeing. What this record adds to them is the reference machine above, which
0004 referred to without naming.

## What holds below the reference machine

A photographer on a five year old laptop gets something that degrades in a
stated way, not something that fails. The statement is the point: degradation
nobody wrote down is indistinguishable from a bug.

Below the reference machine, the interactive targets are held by spending
quality rather than by dropping frames. The grid may draw placeholders for
longer and may draw previews at a lower resolution, and the comparison may take
longer to reach one screen pixel per image pixel. The frame budget itself is not
relaxed, because a surface that drops frames is the thing the budget exists
against, and a surface that shows a coarser thumbnail on time is still usable.

The batch targets are not held. Import throughput and catalogue open time scale
with cores and with disk, and this record makes no promise about either below
the reference machine. What it does require is that the surface says what it is
doing rather than appearing stalled, which is issue #94.

A machine with less memory than the reference is the case where degradation
turns into failure, and the preview store budget in issue #44 is where that is
handled. This record does not set a memory floor, and that absence is deliberate
rather than forgotten: the floor is a measurement nobody has taken.

## Release conditions and aspirations

A target is a release condition when a photographer meets it in the first minute
of ordinary use of what that release ships. Everything else is an aspiration,
and an aspiration missed does not stop a release.

Release conditions for the first release: opening a catalogue, scrolling the
grid, filtering the whole library, comparing two frames at full resolution, and
importing a corpus. Those five are the whole of culling an archive that already
exists, which is what the first release is.

Aspirations for the first release: the two tethering figures. They are not weak
targets, they are conditions for the release that ships tethering, which is M6
and not the first one. Placing them here rather than dropping them keeps them
from being invented at release time.

The split is drawn at what the release ships rather than at what is hard,
deliberately. A target moved to the aspiration column because it turned out to
be difficult is a bar lowered, and the record would have to say so in those
words.

## What this record does not decide

- How any of these is measured in practice. That is issue #25, and the harness
  is the authority for the method once it exists.
- Whether a check refuses a regression against these numbers, and whether that
  check blocks. That is issue #107, which reads its numbers from this record
  rather than restating them.
- What the culling signals cost in time. Issue #59 owns that, and a signal
  budget is not a frame budget.
- Whether there is a surface at all. That is entry 5 of issue #13 and it is
  open. Four of the six targets above are about a surface, and if that entry is
  settled against one, this record loses those four rather than surviving with
  them reinterpreted.

## What would reverse it

The corpus in issue #4 turns out not to represent the load, so a number met on
it is met on nothing. Then the corpus is fixed and these targets are re-set
against the fixed one.

The catalogue measurement in issue #5 shows the filter target is unreachable at
100,000 rows on the reference machine by any engine measured. Then this record
is corrected with the reachable number and the reason, before the code is
written rather than after it is benchmarked.

The reference machine stops representing what a photographer owns. It is a
specification with a date on it, and this record is where it is replaced rather
than quietly reinterpreted.

## Means

This record is Markdown in `docs/decisions/`, which is the means issue #12 names
in its own scope line. It adds no language, runtime or dependency to the tree.
The arithmetic it rests on carries the command that produced it, and every other
number in it is declared as a target rather than as a measurement, which is the
one thing a document can be trusted to do.
