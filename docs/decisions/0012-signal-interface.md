# 0012 The shape every culling signal has

Status: accepted
Date: 2026-08-08
Issue: #50

## The question

A signal takes a photograph, or a group of them, and produces something. Six
things read that something: the ranking, the scheduler, the surface, the
explanation shown to the operator, the catalogue that stores it, and the
evaluation that judges it. Each one could be written against each signal
separately, and then adding the eighth signal would be a change in six places.

So the shape is settled here, before any signal exists, and every signal is
built against it.

## The decision

`crates/culling/src/signal.rs` holds the shape. `crates/culling/src/runner.rs`
holds the one route a signal is called through, and refuses a call or a result
that disagrees with what the signal declared about itself.

Nothing in this record decides what any signal measures or where any threshold
sits. Which signals exist at first release is
`docs/decisions/0006-signal-sources.md`, and every operating point is chosen
where its signal is built.

## What a signal declares before it is called

A name, a version, its scope, what it needs, what it costs, what it requires of
the machine, and what its values look like. All seven are read before anything
runs, and each is there because something downstream cannot work without it.

The **name** is refused rather than normalised where it is not lower case with
single hyphens. A signal stored under two spellings is two signals to everything
that reads the catalogue afterwards, and quietly correcting the second spelling
is how both end up written.

The **version** and the interface version travel with every value, which is the
next section.

The **scope** is one photograph or a group. It is declared rather than inferred
from how the signal was called, so a mismatch is something to refuse rather than
something nobody notices.

**What it needs** is metadata only, a frame no smaller than a stated longest
edge, or the full frame. A signal states the smallest size it is honest at
rather than the size it would prefer, because the scheduler pays for the
difference on every frame of a shoot. How pixels reach this module is the
decoding work in M3 and is deliberately not decided here: the interface names
what a signal needs, not how it arrives.

**What it costs** is a declared budget per photograph rather than a measurement.
The scheduler reads it before anything has run, so it cannot be what the last
run took. Issue #59 is where the declared figure and the measured one are
compared, and a signal whose declaration is wrong is a defect in that signal.

**What it requires** is whether an accelerator is unused, useful or necessary,
and which model it loads if it loads one. Declared for the same reason: a signal
that discovers mid-import that it needs a device the machine does not have has
already spent the import.

**What it produces** is a number between two stated ends, or one of a fixed set
of names. This is what makes a stored value comparable with another value from
the same signal, and it is what the runner holds the signal to.

## What a value carries

Every value carries which signal made it, which version of that signal, and
which version of this interface. A value in a catalogue outlives the code that
made it. Without an origin it cannot be compared against a value from a
different version, and it cannot be dropped when that version is found to be
wrong, which is the repair every signal will eventually need.

The interface version starts at 1 and moves only when a value written under the
previous number can no longer be read as this shape. A field added that every
reader may ignore is not that. A field removed, retyped or given a new meaning
is.

## Having no opinion

A signal returns a measurement or it returns a silence, and the silence carries
no value at all.

That is the load-bearing decision in this record.
`docs/decisions/0005-verdicts-and-error-cost.md` says most frames in most shoots
are neither obviously keepers nor obviously failures, that the honest output
there is an ordering and a silence, and that this is the expected case rather
than a failure to produce one. An interface where silence is a low number, or a
low confidence, or an absent field, is an interface where every reader has to
remember which of those means nothing was said. One of them eventually forgets,
and the frame it forgets on is reported as measured.

The silence says why it is silent, and the three reasons are kept apart: nothing
worth remarking on, the thing this signal looks for is not in this photograph,
and it could not answer here. The first two are ordinary answers and the third
is a failure. Reading the third as the first is how a shoot appears to have been
judged when part of it was not.

## What the operator is shown

A measurement carries its evidence, and evidence carries a region and a
sentence. Neither is optional, because
`docs/decisions/0005-verdicts-and-error-cost.md` says a flag whose evidence
cannot be shown is a flag that is not raised.

The region is in fractions of the frame rather than in pixels. The signal may
have looked at a reduced frame and the surface may be showing a preview of some
other size, and a box in pixels would be right only for the one size it was
measured at. A region that does not lie inside the frame is refused, because it
cannot be placed on any preview.

## Structural rules

Three structural rules are stated here. Two are refused by a test and one is
held by the compiler.

**A signal is called at the scope it declared, and with at least what it said it
needs.** Refused by `crates/culling/src/runner.rs` and proven by
`crates/culling/tests/signal_interface.rs`, in the `scope` and
`what_it_was_handed` modules. Removing the check makes those tests red, which is
how it was checked rather than assumed.

**A value a signal returns is one it declared it produces, and its evidence can
be shown.** Refused by the same module and proven by the `what_it_returned`
tests, whose fixtures sit one step outside each boundary and whose neighbours
sit exactly on it.

**Nothing can read a value out of a silence.** Held by the shape of `Opinion`
rather than by a check: the arm carrying no opinion carries no measurement, so
code reaching for one does not compile. No test here proves that, and proving it
would take a harness that compiles a program and expects the compilation to
fail, which is a dependency this tree does not carry. It is registered as
unenforced in `docs/architecture-rules.md`, with the same reason, rather than
left to look like a tested property.

One further rule is not stated here and belongs elsewhere. That the culling
module never reaches the session, so a signal cannot be tuned to please the
operator, is declared on the `culling` line of `crates/module-boundaries.txt`
and refused by `crates/lichttisch/tests/module_boundaries.rs`.

## What this record does not decide

- What any signal measures, and where its operating point sits. Each is decided
  where the signal is built.
- How a group is formed. That is issue #56, and this record only says that a
  signal may declare it looks at one.
- How pixels reach a signal. That is M3.
- How a ranking combines several signals into an order. That is issue #58, which
  reads what is defined here.
- What the surface does with a region. That is issue #67.

## What would reverse it

- A signal cannot be written against this shape without widening it in a way
  that makes the silence and the measurement readable as one thing. The
  interface is the thing to change, in a new record naming this one, rather than
  the signal being allowed around it.
- The catalogue schema, when it lands under issue #27, cannot store an origin
  per value at an acceptable cost. That reopens how provenance is carried rather
  than whether it is.
- Measurement under issue #59 shows that a declared cost cannot be stated
  usefully ahead of a run, which would make the scheduler read something else
  and would remove one of the seven declarations.

## Means

The interface is Rust in `crates/culling`, which is the module
`docs/layout.md` gives to the culling signals and the one
`crates/module-boundaries.txt` already declares. It is the means the tree is
written in and it adds no language, runtime or dependency: the whole of this
change is the standard library.

It carries the three rules the tree is held to. A property a machine can refuse:
the runner refuses four things and each refusal names the signal. A proof that
runs: each refusal has a fixture that trips it and a neighbour that must stay
green, and each was checked by removing the guard and watching the suite go red
rather than by assertion. A claim carrying the command behind it: the tests are
run by the suite that already exists, `cargo test --locked --workspace`, and no
parallel apparatus is added.

The scope line of issue #50 names `crates/signals/`, and this landed in
`crates/culling` instead. No such directory exists in this tree. The module
holding the culling signals is `crates/culling`, named in `docs/layout.md` and
declared in `crates/module-boundaries.txt` before this issue was written. The
path in the issue is the older name for the same module rather than a second
one, and creating the second would have been the change nobody argued.
