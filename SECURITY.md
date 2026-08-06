# Security policy

This software parses files that arrived on removable media and takes bytes over
a cable from a device somebody else built. Both of those are untrusted input.
Somebody will eventually find something, and when they do there has to be a
route that is not a public issue.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting, at
[Report a vulnerability](https://github.com/iderex/lichttisch/security/advisories/new).
That is the preferred route and it is the only one that is private from the
first message.

It is enabled on this repository:

    gh api repos/iderex/lichttisch/private-vulnerability-reporting
    {"enabled":true}

Do not open a public issue for a suspected vulnerability. If you have already
opened one, report it privately as well and say in the private report that a
public issue exists, so the two can be handled together.

## What a reporter can expect

Each period below runs from the moment the report arrives, in calendar days.

| Step | Period | What it means |
| --- | --- | --- |
| Acknowledgement | 3 days | A human has read the report and it is not lost. Nothing about the finding is judged yet. |
| Assessment | 14 days | Whether the report is reproduced, what it affects, and how serious it is judged to be, with the reasoning. Where it is not reproduced, what was tried and what was missing. |
| Decision | 45 days | Whether a fix is being made, what it is, and when it is expected. Where the answer is that no fix will be made, the reason is given rather than the report being left to lapse. |

If a period is going to be missed, the reporter is told before it expires
rather than after, and told what is holding it up.

Disclosure is coordinated with the reporter. The default is that the advisory
is published once a fix is available, and that the reporter is credited unless
they ask not to be.

## Which versions receive fixes

There is no release yet. Until there is one, the default branch is the only
thing that exists, and a fix lands there.

Once there is a release, the rule is the current release and the default branch.
An older release receives a fix only where that is stated for that release, in
that release's own notes, rather than being assumed from this sentence.

This section is written so it stays true before the first release rather than
naming a version table that does not exist. When the first release lands, this
section is replaced with the real answer rather than left to be read
generously.

## In scope

These are the classes this policy is about, and they are the places where bytes
this project did not create reach code this project does control.

- File parsing. Raw files, rendered files, and anything else read from a card,
  a disk or a watched folder. A crafted file that causes memory corruption, an
  out-of-bounds read, an unbounded allocation, or execution of anything, is in
  scope.
- Metadata parsing. Embedded capture metadata and sidecar files, including
  sidecars written by another application. A sidecar is text somebody else
  produced and it is parsed like any other untrusted input.
- The camera transport. Bytes arriving over a cable from a device, including a
  device that is lying about what it is or how much it is about to send.
- The catalogue file. A crafted or corrupted catalogue that causes anything
  worse than a refusal to open it, and anything that lets one catalogue reach
  outside its own directory.
- Any path where this software writes outside the set of locations it declares
  it may write to, or modifies a photograph. This project's rule is that it
  never deletes, never moves and never writes into a photograph, and a way to
  make it do so is a vulnerability rather than a bug.
- Anything that causes photograph content, or a value derived from it, to leave
  the host.

## Out of scope

Named with the reason, so a reporter does not spend their time on something
that will be declined.

- Denial of service that needs the operator to feed the software their own
  files deliberately. The operator already controls what they import, and a
  large or awkward file making an import slow is a performance issue rather
  than a security one. A crafted file that hangs or exhausts memory during an
  automatic watched-folder import is in scope, because nobody chose that file.
- Vulnerabilities in a dependency, reported to us rather than upstream. Report
  those upstream. Tell us as well, and we will pin, patch or drop the
  dependency, but the fix belongs where the code is.
- Findings that require an attacker who already has the operator's account on
  the machine. Somebody with that already has the photographs, so nothing this
  software does can be the weak link there.
- Missing hardening that has no route to an effect, reported as a finding on
  its own. A compiler flag or a header that is absent is worth telling us about
  and it is worth fixing; it is not treated as a vulnerability unless there is
  a path from its absence to something happening.
- The security of another application this project interoperates with. Report
  those to that project.
- Anything about a build somebody produced themselves with a component this
  project does not ship.

## Please do not send us photographs

A report is never required to include an exploit against anybody's real
photographs, and one is never asked for. Do not send a photograph of a person
in order to demonstrate a finding.

Send a synthetic reproducer instead: a generated or hand-built file that
carries the structure the finding needs. If the finding depends on a real
camera's output, a frame of a wall, a test chart or an empty room carries the
same file structure as a frame of a person and is a complete reproducer.

If a finding genuinely cannot be shown without a specific real file, say so in
the report and describe the structure instead. We would rather work from a
description than hold somebody's photograph, and a report without a file is not
declined for that reason.
