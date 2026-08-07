# 0005 What a culling verdict is, and what a wrongly culled keeper costs

Status: accepted
Date: 2026-08-07
Issue: #8

## The question

This software will produce opinions about photographs. Somebody has to say what
those opinions are, what weight they carry, and what it costs when one of them
is wrong. That is settled here, before a signal exists, because the answer
decides what the software is allowed to do rather than how it is tuned.

## The decision

The machine ranks and flags. The photographer decides. The two are different
kinds of value, they are stored differently, and nothing the machine produces
ever becomes a decision on its own.

## The vocabulary

What the machine produces, for one frame:

- a **signal value**, a number or a category computed from pixels or from the
  metadata the camera wrote, carrying the version of whatever produced it
- a **flag**, a statement that one named signal crossed one stated threshold,
  carrying the reason and the evidence for it
- a **rank**, an ordering of frames inside a group, carrying what the ordering
  was made of

What the photographer produces:

- a **rating**, a **colour label**, a **rejection**, and the descriptive
  metadata they write

Which of the two any stored value is, is not a matter of reading its name. It
is the class the value sits in, and `docs/decisions/0010-catalogue-schema.md`
carries those classes. Everything in the first list is **derived**: computed
here, droppable, rebuildable, and never anything the photographer set.
Everything in the second is **cached**: a copy of what the sidecar or the file
says, never the authority, corrected from the source when the two disagree.
`docs/decisions/0002-integration-shape.md` is where that authority rule is
argued.

So there is no column anywhere in which a machine opinion and a photographer's
decision could be confused for one another, because they do not share a class,
and a value that would have to sit in both is a design error rather than a
column to add.

The word **verdict** in this project means the photographer's decision and
nothing else. A flag is not a small verdict and a rank is not a provisional
one.

## The asymmetry

The two errors are not two entries to average.

A frame kept that should have been dropped costs a second of the
photographer's attention. They look at it, they move on.

A frame dropped that should have been kept can cost the only usable frame of
something that will not happen again. There is no second exposure of the moment
somebody's expression changed, and no amount of later care recovers it.

An accuracy figure adds those two together and divides. It is the wrong
instrument for this problem, and it is the instrument most likely to be quoted,
which is why this record names it rather than leaving it out.

## The four consequences

**Nothing this software computes removes a file, moves a file, or writes a
rejection the photographer did not make.** The signals produce a rank and a
flag with a reason attached. That is the whole of what they produce. A frame
this software is confident about and a frame it has never looked at are the
same thing on disk.

**An operating point is chosen on how many keepers survive it, and it is
reported that way.** Where a threshold has to be picked, the sentence that
picks it says what fraction of keepers survive it, and that sentence comes
before any sentence about time saved. Time saved is what the tool is for; keeper
survival is what it is allowed to cost.

**Every flag carries its reason and the evidence for it.** A photographer
disagreeing with this software disagrees with a specific claim about a specific
frame, at the region of the frame that produced it, and not with the tool in
general. A flag whose evidence cannot be shown is a flag that is not raised.

**The operator can move the operating point, and moving it states the cost in
the same terms as the gain.** A control that says only what it buys is a
control that hides what it spends. Both numbers, in one sentence, at the point
the operator moves it.

## What governs a release

One figure governs, and it is the one that is expensive to be wrong about.

**Keeper survival at the shipped operating point.** The fraction of frames the
photographer would have kept that this software does not flag for dropping.
Issue #61 builds the set it is measured on and issue #62 is where it is
published.

Reported beside it, and never in place of it: the fraction of frames flagged,
the time a session takes with and without the tool, and the per-signal figures
that show which signal contributed which part of the error.

A headline accuracy figure is not among the governing ones, and it is not
reported as a summary of the others. Where one is asked for, the answer is the
keeper survival figure and the flagged fraction, both, because those two are
what the single number was hiding.

## When it has no confident opinion

Most frames in most shoots are neither obviously keepers nor obviously
failures. The honest output there is an ordering and a silence: the frames are
ranked inside their group, and no flag is raised.

That is the expected case and not a failure to produce one. A tool that flagged
something on every frame would be reporting its own threshold rather than
anything about the photographs. Nothing in the surface, the logs, or the
published figures treats an unflagged frame as a gap, and a release is not
judged on how many frames it had an opinion about.

## What this record does not decide

It does not decide the thresholds. Every operating point is chosen where its
signal is built, against a measurement, and issue #62 is where the error that
costs the most is published.

It does not decide what the surface shows. Issue #67 owns showing the region
that produced a flag and issue #68 owns disagreeing with it.

It does not decide whether this software may ever remove a file behind an
explicit, separately confirmed action. That is entry 6 of issue #13, it is the
maintainer's, and it is open. This record is written on the answer the plan
already assumes, which is that it never does.

## What would reverse it

- The maintainer answers entry 6 of issue #13 in a way that permits removal.
  The vocabulary above survives that answer, because a separately confirmed
  removal is still the photographer's decision rather than the machine's. What
  changes is the first consequence, and it changes by gaining a door with its
  own confirmation rather than by softening the sentence.
- Measurement shows that keeper survival cannot be reported honestly at any
  operating point for a given signal. The answer is to publish the figure and
  drop that signal from the default set, which is issue #62, rather than to
  change the governing metric to one the signal passes.
- A photographer's own trial shows that ordering plus silence is unusable in
  practice and that the tool has to commit. That reopens this record rather
  than being absorbed by it, because committing is what the first consequence
  refuses.

## Means

This record is Markdown in `docs/decisions/`, which is the means issue #8 names
in its own scope line. It adds no language, runtime or dependency to the tree.
It enumerates no set that lives elsewhere: the storage classes are named and
`docs/decisions/0010-catalogue-schema.md` is the authority for their contents,
and each threshold is owned by the issue that builds its signal.

Nothing here is refused by a check. The four consequences are properties of
code that does not exist yet, and the issues that build it are where each one
becomes something that can fail. That is stated so this record is not mistaken
for a mechanism.
