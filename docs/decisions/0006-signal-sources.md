# 0006 Where the culling signals come from at first release

Status: accepted
Date: 2026-08-06
Issue: #9

## The question

A culling signal can be a measurement over pixels, a pretrained model somebody
else published, or a model trained in this repository. The three have different
quality and different legal cost, and they pull against each other. Which of
them does the first release use?

## The decision

The first two only. Nothing is trained in this repository before the first
release. The culling quality that buys is measured and published rather than
claimed.

## Per signal

Measured. No model, no weights, nothing learned. These are arithmetic over
pixels and over the metadata the camera wrote.

| Signal | Issue | What it is |
| --- | --- | --- |
| Exposure and clipping | #51, #57 | Histogram statistics over the decoded frame |
| Global sharpness | #51 | Gradient energy over the frame |
| Motion blur | #57 | Directional gradient statistics over the frame |
| Burst and session grouping | #56 | Capture timestamps and camera metadata |
| Near-duplicate candidates within a group | #56 | Similarity over reduced-resolution frames |
| Frames that are not a matter of taste | #57 | Truncated file, absent or impossible exposure, uniformly black or white frame |
| Ranking a group | #58 | A stated function over the signals above, with its weights written down rather than learned |

Pretrained, permissively licensed, used as published. Face detection and face
landmarks are a widely distributed capability, and neither is a model this
project would improve by making its own.

| Signal | Issue | What is pretrained | What is computed here |
| --- | --- | --- | --- |
| Face and subject location | #53 | Face detector | Nothing; the boxes are used as published |
| Focus on the subject | #54 | The same detector, for the region | The sharpness measure over that region, which is measured rather than learned |
| Eyes open or closed | #55 | Face landmark model | An openness ratio over published landmark coordinates, with an operating point stated in #55 |

Trained here. Nothing, before the first release.

## The pretrained models, their versions and their licences

Two sources are carried, one primary and one fallback, so a single upstream
change cannot remove the capability. Issue #60 is what pins an exact artefact
and records its terms before anything ships; this record names where the models
come from and on what basis.

Primary, MediaPipe, for both face detection and face landmarks:

    gh api repos/google-ai-edge/mediapipe --jq .license.spdx_id
    Apache-2.0
    gh api repos/google-ai-edge/mediapipe/releases --jq '.[0] | "\(.tag_name)\t\(.published_at)"'
    v1.0.0	2026-07-28T22:13:25Z
    gh api 'repos/google-ai-edge/mediapipe/contents/LICENSE?ref=026623b739fd9dabab6751eee53b6f1dec092bc2' --jq .content | base64 -d | head -3
                                     Apache License
                               Version 2.0, January 2004
                            http://www.apache.org/licenses/

Fallback, OpenCV, for face detection:

    gh api repos/opencv/opencv --jq .license.spdx_id
    Apache-2.0
    gh api repos/opencv/opencv/releases --jq '.[0] | "\(.tag_name)\t\(.published_at)"'
    4.14.0	2026-07-19T11:45:21Z
    gh api 'repos/opencv/opencv/contents/data/haarcascades?ref=4bee6895ebf48f998ab8cfeccf2b5eb56b9dfa3a' --jq '[.[].name] | length'
    17

What those commands read is the licence of each source repository, and the
version is the latest release tag at the time of writing. They did not read the
terms of an individual model artefact. A weights file distributed alongside a
repository can carry terms of its own that differ from the repository's, and no
such file was opened on this route. Issue #60 is where the exact artefact, its
digest and its own terms are read and recorded, and it is a condition of
shipping rather than of this decision. Until it lands, this project has an
argued source and not a pinned model.

An eye-openness decision computed from published landmark output is a
computation over somebody else's model rather than a model of this project's
own. That distinction is what keeps the pretrained row on the permissive side
of the line, and it is why the openness threshold in #55 is a number this
project states and defends rather than a weight it fits.

## The rule about photographs and training

No photograph reaching this software is used to train anything. Not the
operator's photographs, not a sample sent with a bug report, not a frame that
arrived over a cable.

Where it is enforced today: nowhere, and that is stated rather than softened.
The rule holds at the time of writing only because no training code exists in
the tree, which is a fact about the tree and not a mechanism. Issue #102 is the
check that makes greppable invariants a merge condition, and this invariant
belongs in its declared set. It is not in that set today. Until it is, this rule
is prose, and the paragraph above is the whole disclosure.

Two neighbouring rules cover part of the same ground and neither is a substitute
for it: issue #92 keeps photograph-derived values out of the logs, and issue #95
makes a bug report possible without sending a photograph. Both reduce what could
reach a training set. Neither refuses a training path.

## The quality cost

This answer costs accuracy and the cost is stated rather than buried.

A blink decision derived from landmark geometry is weaker than a classifier
trained for the job. It is weakest exactly where weddings and events live: low
light, partial occlusion, subjects in glasses, and faces at an angle. A
landmark model that was not fitted for eyelid geometry under those conditions
puts its points somewhere plausible and the ratio computed from them is
confident and wrong.

Subject focus inherits the detector's failures. Where no face is found, the
measure falls back to the frame, and a portrait judged on frame sharpness is
judged on the background.

How much weaker is a number this project does not have. Issue #62 is what
produces it, per condition rather than as an average, on the evaluation set
issue #61 makes reproducible by somebody else. Until those land, no figure about
culling quality is published here or anywhere else in this tree.

## What has to be true before training begins here

All of the following, and none of them alone:

- The maintainer has answered entry 4 of issue #13, whether a photograph may
  ever be used to train a model. It is open at the time of writing and nothing
  in this record assumes an answer to it.
- Every candidate corpus has had its terms read and the reading recorded, per
  corpus, with the date and the clause relied on. A research-only clause in a
  face dataset is the ordinary case rather than the exception, and a training
  set assembled without that reading makes this project the route by which
  somebody ends up in breach of terms they never saw.
- There is a record saying where the photographs came from and on what basis,
  and it names a person or an organisation rather than a download location.
- The rule above has a mechanism rather than a sentence, so that "no operator
  photograph was used" is something a check refuses to let become false.
- The quality claim that motivates the training is measured first, so the
  improvement is a number against issue #62 rather than an expectation.

Training later is not ruled out. It is sequenced, and the sequence above is the
whole of it.

## What would reverse it

- The maintainer answers entry 4 of issue #13 in a way that permits training,
  and every condition in the section above is met. The first two pretrained
  signals would then be candidates for replacement, one at a time, each against
  a measured improvement.
- A pretrained model this record depends on changes its terms, or its
  distribution stops. The fallback covers face detection. It does not cover
  landmarks, so a change on that side reopens this record rather than being
  absorbed by it.
- Issue #62 measures the landmark-derived blink decision as too weak to ship at
  any operating point that is honest about its false positives. The answer then
  is not to train quietly; it is to publish the figure, drop the signal from the
  default set, and reopen this record.

## Means

This record is Markdown in `docs/decisions/`, the means issue #9 names in its
own scope line. It adds no language, runtime or dependency to the tree. The
tables classify rather than enumerate a set that lives elsewhere: each row names
the issue that owns the signal, and the issue is the authority for what that
signal does.
