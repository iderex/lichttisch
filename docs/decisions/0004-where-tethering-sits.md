# 0004 Where tethering sits

Status: accepted
Date: 2026-08-06
Issue: #7

## The question

Tethered capture is named in the readme at the level of the professional
incumbent. Where does it sit in the architecture: inside the culling
application, in a process of its own, or nowhere at all?

It is answered before the capture milestone starts, because the answer decides
what that milestone can be tested without.

## The decision

A separate capture process. It owns the camera, writes the frame to disk, and
feeds the catalogue through the same ingest path a card import uses. The camera
transport sits behind an interface, and a fake implementation of that interface,
needing no hardware, exists from the first day so the milestone can be tested on
a machine with nothing plugged into it.

## Position one: a view inside the culling application

The capture code shares a process with the surface.

It buys the shortest path from shutter to screen. There is no serialisation, no
process boundary and no contract to design.

It costs two things that matter more. A camera that stops responding takes the
culling surface down with it, and a camera that stops responding is an ordinary
event on a working shoot rather than a fault. And the surface can then only be
tested with hardware attached, which means the automated checks either skip the
capture path entirely or the checks need a camera, and both of those are worse
than the process boundary.

## Position two: a process of its own

The capture process owns the camera and hands frames to the same ingest path a
card import uses.

It buys a camera that cannot stall the interface, a transport that can be
replaced without touching the catalogue, and one ingest path exercised by both
sources rather than two paths that drift apart. Issue #77 is where the single
path is asserted by a test rather than intended.

It costs a process boundary and a contract across it, both of which have to be
designed rather than assumed. The contract is not free: it has to carry
back-pressure, it has to say what happens to a frame in flight when either side
dies, and it has to be versioned once an operator can upgrade one side without
the other.

## Position three: out of scope

Delegate to the raw developer's own tethering and watch the folder it writes
into.

It buys nothing to build.

It costs the thing the readme promises. The workflow claimed there is shutter to
a ranked, comparable, culled frame. A watched folder delivers a file, eventually,
with no notion of which shutter release it came from and no path for a rank to
travel back. At the latency a working shoot needs, this is not the same feature
with a slower implementation. It is a different, smaller feature.

## The latency this placement is meant to buy

The placement is chosen to keep two numbers reachable. Both are measured from
the shutter release, on the reference machine the benchmark harness names, at
the 95th percentile over a run rather than as a best case:

- the frame is visible to the operator within 1.5 seconds
- the frame is ranked and comparable within 3.0 seconds

These are targets set by this decision. They are not measurements and no
measurement of them exists at the time of writing. Issue #77 is what measures
them, through the benchmark harness in issue #25, and its own definition of done
names this record as the number to measure against. Issue #12 is where the
release performance bar is set, and it may tighten or loosen these two figures;
where it does, this record is the thing that gets corrected rather than the
place the older number keeps living.

The first number is reachable only because the embedded preview is displayed
before the raw is decoded. That substitution is issue #41 and issue #77, and it
is visible to the operator rather than silent, because a photographer judging
focus needs to know they are looking at the camera's own rendering.

## What happens when the capture process dies

The culling surface stays up. This is the whole point of the boundary, so the
behaviour is stated rather than left to whatever the code happens to do.

- The catalogue and every frame already ingested remain available, and the
  operator can keep culling. Nothing about the session depends on the capture
  process being alive.
- The operator sees a plain statement that capture has stopped, naming the
  camera it was connected to and the time of the last frame that reached disk.
  It is not a transient notification, because the operator may be looking at a
  photograph rather than at the status area when it happens.
- The count of frames written since the session started is shown alongside it,
  so the photographer can compare it against the camera's own counter and find
  out whether anything was lost.
- Nothing is retried silently. Reconnection is an action the operator takes, and
  it reports whether the camera came back.
- A frame that was in flight when the process died is either fully written and
  in the catalogue, or absent from both. Issue #77 holds the ordering rule that
  makes this true and the kill test that proves it.

## The redistribution constraint

The transport is held behind an interface for a licensing reason as well as an
architectural one, and the reason is stated here so the capture milestone does
not have to rediscover it.

The open transport reaches a large part of the field with no agreement of any
kind:

    gh api repos/gphoto/libgphoto2 --jq .license.spdx_id
    LGPL-2.1
    gh api 'repos/gphoto/libgphoto2/contents/camlibs?ref=cc81290cf1d2a6101ff71487ac96dde772f9efb9' --jq 'length'
    59

That is 59 camera library directories at that commit. The count is directories,
not camera models, and it is not a supported-camera list: issue #82 is where the
list is derived rather than written.

The vendor kits reach further into the camera, which is where the difference
between adequate tethering and professional tethering actually lives. Each one
arrives with an agreement restricting redistribution and sublicensing, so a
build linking one cannot be shipped the way this project intends to ship. This
repository is AGPL-3.0:

    gh api repos/iderex/lichttisch --jq .license.spdx_id
    AGPL-3.0

So the interface is what keeps the shipped artefact one thing that anyone may
redistribute, whatever is decided later about builds that are not shipped.

This record does not decide which transports ship. Issue #74 chooses the
transport and owes an argued compatibility statement quoting the clause it
relies on, rather than a licence identifier copied from a listing, which is the
obligation entry 1 of issue #13 places on it. Whether a build linking a vendor
kit may ever be produced at all is entry 2 of issue #13, it is open, and nothing
here assumes an answer to it. The interface is chosen so that every answer to
that entry remains available: never link one, permit an operator-produced build,
or permit a shipped build per vendor.

## What would reverse it

- The process boundary proves to cost more latency than the placement can pay
  for. Concretely: issue #77 measures the frame reaching the operator later than
  1.5 seconds at the 95th percentile, and the shortfall is traced to the
  boundary rather than to decoding, disk or the transport. A shared process
  would then be back on the table, with the fault isolation lost on purpose and
  written down as lost.
- A transport arrives that cannot be driven from a process that does not own the
  surface, so the boundary is not implementable rather than merely expensive.
- The maintainer settles entry 5 of issue #13 against this project having a
  surface of its own. Tethering into a headless engine is a different question
  from the one this record answers, and this record would be reopened rather
  than reinterpreted.

## Means

This record is Markdown in `docs/decisions/`, the means issue #7 names in its
own scope line. It adds no language, runtime or dependency to the tree, and the
claims it makes about another project carry the commands that produced them.
