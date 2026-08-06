# 0010 The catalogue schema, and what a photograph is to it

Status: accepted for the model, incomplete for the schema
Date: 2026-08-06
Issue: #27

## The question

Before rows are written, the thing a row describes has to be defined. A raw file
and its out-of-camera JPEG are two files and one photograph. A raw file copied
to two drives is one photograph in two places. A file edited elsewhere and
written back is arguably a second photograph. A burst of eight frames is eight
photographs that only mean something together.

Each of those is answered below rather than skipped.

## The four things

**A file.** A sequence of bytes. Its identity is the digest of those bytes, and
nothing else. A file's path is where a copy of it was found, not what it is.

**A copy.** One file found at more than one location. Two drives holding the
same bytes are one file and two copies. A copy carries where it was seen, when
it was last seen there, and whether it is still there. Nothing about the
photograph is stored on a copy.

**A photograph.** What the photographer means when they say "that shot". One
shutter release, however many files came out of it. A raw and its
out-of-camera JPEG are two files belonging to one photograph, and each is a
rendition of it.

**A group.** A set of photographs that mean something together. Two kinds, and
they are not the same kind of fact. A session is a stretch of shooting bounded
by a gap, and it is a convenience for browsing. A burst is a run of frames the
camera took in a row at the camera's own rate, and it is what the culling
comparison operates on. Both are derived, both are rebuildable, and neither is
something the photographer set.

## The identity rule

What makes two files the same photograph:

They carry the same capture identity. Capture identity is the triple of camera
body identifier, capture instant including its sub-second part, and the camera's
own frame number, read from the embedded metadata of each file.

What makes them different:

Anything else. Two files with no capture identity between them are two
photographs, even when they look identical, because a resemblance is not a fact
about what happened at the camera.

Three consequences, stated so they are not rediscovered as bugs.

Where capture identity is absent, each file is its own photograph. A scanned
image, a file whose metadata was stripped by another tool, and a render that
carries nothing about its origin all fall here. This is a deliberate
under-grouping: the software says less rather than guessing, and #29 is where
reading that metadata without decoding the image is built.

Where capture identity is incomplete, it is not used. A body identifier and a
second-resolution timestamp with no sub-second part will collide across a burst
at eight frames per second, and a rule that groups two different frames into one
photograph is worse than one that groups nothing. The partial value is recorded
as observed and the photographs stay separate.

Two cameras can produce the same triple. Two bodies of the same model with the
same serial reported, shooting in the same second with the same frame counter,
is rare and it is not impossible. The rule is stated as what it is, a strong
heuristic over what the camera wrote, rather than as a guarantee, and #30 is
where recognising the same photograph arriving twice is built and where this
rule is either demonstrated or found to be too weak.

## The awkward cases, answered

**A raw and its out-of-camera JPEG.** Two files, one photograph, two renditions.
Each file keeps its own digest, its own size and its own format. The photograph
carries which rendition is the primary one, which is derived from a stated rule
and not something the photographer set.

**A raw copied to two drives.** One file, one photograph, two copies. Ranking it
in one place ranks it in the other, because the rank was never on the copy. A
copy that disappears leaves the photograph and its verdict intact, and #31 is
where files moving, vanishing and coming back is held.

**A file edited elsewhere and written back.** A new file, always, because the
bytes are different and a file is its bytes.

Whether it is a new photograph depends on what the file says about itself, and
on nothing else. If it carries the original's capture identity in its metadata,
it is another rendition of the same photograph. If it carries a derived-from
identifier or the original's digest, that relationship is recorded as a
relationship the file asserted. If it carries neither, it is a new photograph
that happens to look like one already in the catalogue, and the software says
so by saying nothing.

No lineage is inferred from a filename, a timestamp or a resemblance. That rule
is `docs/decisions/0007-scope-boundary.md` and it is the same rule here for the
same reason: a guessed parent is confident and the photographer cannot check it.

**A burst of eight frames.** Eight photographs, one group. The group is derived
and rebuildable. Each frame keeps its own verdict, because the photographer
chooses one frame out of the burst and not the burst. #33 builds the grouping
and #56 finds which frames within it are the same picture.

## Every column, classified

The authority rule in `docs/decisions/0002-integration-shape.md` has to be
visible in the shape of the schema rather than only in a document, so every
column carries one of four classes and the class is part of the column's
declaration.

**Observed.** Read from the file. True because the file says so. Re-reading the
file reproduces it.

- file: digest, byte length, format, container
- file: capture instant, sub-second part, camera body identifier, frame number
- file: camera make and model, lens, focal length, aperture, shutter, sensitivity
- file: orientation, pixel dimensions, colour space as declared
- copy: path, volume identifier, modification time as the filesystem reports it

**Cached.** A copy of what the sidecar or the file's own metadata says the
photographer decided. Never the authority. When it disagrees with the source,
the source wins and the cache is corrected.

- photograph: rating
- photograph: colour label
- photograph: rejection flag
- photograph: title, caption, keywords, and any other descriptive metadata the
  photographer writes

Each cached column carries the source it was read from and when it was read, so
a disagreement can be resolved rather than argued about. #83 is where the
sidecar is read and written and #88 is where a change made outside this
software is reconciled.

**Derived.** Computed by this software from observed values and from pixels.
Droppable and rebuildable without touching anything else.

- photograph: which rendition is primary
- photograph: every culling signal value, with the version of what produced it
- photograph: preview references, at each size
- photograph: group membership, for sessions and for bursts
- photograph: duplicate and near-duplicate relationships, with what decided them
- group: rank order within a burst, and what the ranking was made of

**Operator.** Set by the operator inside this software and nowhere else. If a
value is one the photographer could also set in another tool, it does not belong
here, and putting it here is the mistake the authority rule exists to refuse.

- photograph: the operator's disagreement with a signal, recorded against the
  signal rather than replacing it, which is #68
- session state: where a culling session had reached, which is #70
- import bookkeeping: when a file was first seen by this software

Import bookkeeping is deliberately in this class rather than in derived. It
cannot be rebuilt from the files, because "when this software first saw it" is
not a fact any file carries. It is a fact this software created, and losing the
catalogue loses it. That is a small, stated exception to "delete the catalogue
and only derived work is lost", and it is stated rather than allowed to make the
larger claim slightly untrue.

## What this deliberately does not model

- Edit history, develop settings, adjustment stacks and versions. Those belong
  to the raw developer, which owns them, and this project holds no opinion about
  them. A file produced by an edit is a file here, and nothing more.
- Who is in a photograph. Faces are found and measured, and no identity is
  attached to a face. #113 is where what is computed, kept and deleted about a
  face is stated.
- Where a file ought to live. This software does not move files, so it has no
  model of a destination, a folder policy or an archive tier.
- Deletion. Nothing here records an intent to remove a file, because this
  software never removes one, which is #71.
- Anything crossing a host boundary. Whether the catalogue may ever leave the
  host is entry 3 of #13, it is open, and no column here assumes an answer to
  it.
- Print, publication and delivery state. That is a different tool's job.

## What is not decided here, and what is not yet built

This record defines the model. It does not choose the storage engine, which is
#5 for the measurement and #6 for the choice, and nothing above depends on the
answer: the four things and the four classes are the same whichever engine
holds them.

Two of this issue's conditions are not met by this record and cannot be met
until the engine and the skeleton exist.

The schema is not yet expressed as migrations. #27 asks that it live in the tree
as migrations from the first version, so that version one is already a
migration rather than a description. This record is a description, deliberately,
because a migration has to be written against an engine and there is not one
yet. When migration one is written, the columns above are what it declares, and
this record becomes the reasoning behind it rather than a second copy of it.

There is no test asserting that the derived columns can be dropped and rebuilt
without touching the cached or observed ones. That test needs a schema, a
catalogue and a suite, and none of the three exists. It is the test that keeps
the classification above honest, so it is a condition of #27 rather than a later
improvement.

## What would reverse it

- #30 finds the capture-identity triple too weak in practice: renditions that
  should group do not, often enough to matter, on real camera output. The
  identity rule would then be widened, and it would be widened with a
  measurement rather than with an intuition.
- The interchange decision, #10, finds that the sidecar cannot carry a value
  this record classifies as cached. That value would then be either operator or
  absent, and neither of those is a change to make quietly.
- Entry 5 of #13 is settled against this project holding a catalogue at all.

## Means

This record is Markdown in `docs/decisions/`, which is the means #27 names in
its own scope line for this half of the work. The other half, the migrations,
takes its means from #2 and #6 and is not chosen here.
