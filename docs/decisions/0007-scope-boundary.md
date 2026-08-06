# 0007 The boundary to the generative editing project

Status: accepted
Date: 2026-08-06
Issue: #11

## The question

There is a public sibling repository, `iderex/retusche`, that generates pixels:

    gh api repos/iderex/retusche --jq '{license: .license.spdx_id, private: .private, description: .description}'
    {"description":"Self-hosted generative photo editing backend: inpainting, outpainting and object removal for photo libraries","license":"AGPL-3.0","private":false}

The two projects share a subject and a photo library. Writing the boundary down
once, in a place a reader will find it, is cheaper than answering the question
every time it comes up.

## What this project does

It selects and it catalogues. Concretely: it imports folders and tethered
captures, it holds a catalogue of what it found, it computes culling signals
over the frames, it presents them for a decision, and it writes the decision
where the photographer's existing tools will read it.

Every pixel it produces is a preview of a pixel that already existed. Decoding a
raw file, resizing it and colour-managing it are renderings of what the sensor
recorded.

## What this project deliberately does not do

It does not generate pixels. No inpainting, no outpainting, no object removal,
no upscaling that invents detail, no generative fill of any kind.

That work is done in `iderex/retusche`. It is not a gap here waiting to be
filled; it is a different project that already exists.

The exclusion is not only about scope. The moment this project generates pixels,
every claim it makes about being a selection tool changes, and the transparency
duties that attach to generated images attach to it too. Issue #115 is where
what this project is not for is stated for an operator, and this record is what
that statement rests on for this particular boundary.

## No code dependency, in either direction

Neither project calls the other. Neither catalogue is the other's.

This tree names the sibling nowhere outside this record and the one sentence in
the readme that links it:

    git grep -ril 'retusche' -- .
    README.md
    docs/decisions/0007-scope-boundary.md

The sibling's dependency manifests do not name this project:

    gh api 'repos/iderex/retusche/contents/pyproject.toml?ref=aaecf3fb56126f07025a7b41ec59fb892f999e49' --jq .content | base64 -d | grep -n -i 'lichttisch'
    gh api 'repos/iderex/retusche/contents/uv.lock?ref=aaecf3fb56126f07025a7b41ec59fb892f999e49' --jq .content | base64 -d | grep -c -i 'lichttisch'
    0

The first of those two prints nothing and exits 1, which is grep finding no
match. What was read is the two manifests at that commit, not the whole tree, so
the claim is bounded to declared dependencies.

Adding a dependency in either direction is a decision rather than an
implementation detail. It would be argued in a record of its own, in both trees,
before the code that needed it existed. A call from a culling session into a
generative backend is not a convenience: it changes what this software is, and
it should cost an argument.

## How an edited derivative is treated

A photograph this project catalogues may later be edited by the sibling, or by
any other editor. The result comes back into the library as a file.

It is treated as a new file. It is imported like any other file that appeared in
a watched folder, it gets its own row, and nothing about it is special because
of where it came from.

Its relationship to the original is recorded when the metadata says so, and left
unstated when it does not. There is no inference here. If the file carries a
provenance field, a derived-from identifier, or the original's digest, that link
is recorded as a relationship the file asserted. If it carries nothing, the file
stands alone, and this software does not guess a parent from a filename, a
timestamp or a visual resemblance. A guessed lineage is worse than none, because
it is confident and it is unfalsifiable from where the photographer is standing.

Issue #30 is where arriving twice is recognised, and it decides identity from
the file. A generated derivative is not the same photograph as its original
under any identity rule this project adopts, because the pixels differ by
construction. Issue #27 is where the relationship between a photograph, a file
and a copy is defined, and the derivative relationship above is one it has to
carry.

## What this record does not decide

- Whether this project ever displays or acts on a provenance signal, beyond
  recording it. That is not decided here.
- What the sibling does. This record is written from this side of the boundary
  and binds nothing there.

## What would reverse it

- A maintainer decision to make generative editing part of this project. That is
  not on issue #13 today, and it would be a new entry there rather than a change
  made in passing.
- The two projects needing to share a component that is neither selection nor
  generation, a colour pipeline for example. That would be a third thing both
  depend on, which is not either project depending on the other, and this record
  would be amended to say so rather than quietly stretched.

## Means

This record is Markdown in `docs/decisions/`, the means issue #11 names in its
own scope line. The readme carries one sentence and a link rather than a copy,
so there is one place the boundary is stated and no second place to drift from
it.
