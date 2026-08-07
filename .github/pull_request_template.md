<!--
The pull request template (#24). Keep it filled in rather than deleted: four
short answers, and the whole review starts from them. Everything about a change
belongs in this body, including a later correction to it.

This file is what the hygiene check reads (#137). The reference line below and
the headings below it are the shape it requires, so filling this in cannot fail
it, and changing what a body owes is a change here rather than in a workflow.
A heading may be elaborated; the template's words have to stay at the front of
it. What the check cannot judge is at the top of tools/pr-hygiene/src/main.rs.
-->

Closes #

## What failure this prevents

<!-- Not what changed, which the diff already says. What goes wrong without it. -->

## The means, and why it fits

<!--
One sentence naming what the change is made of, and why that is right for this
artefact rather than for the last one.
-->

## What was run, and what came back

<!--
Paste the output. There is no single command that runs every gate locally and
two of them have no local form at all, so "not run locally" is a correct answer
where it is true. Write it rather than leaving this empty; an empty section
reads as a run that passed.
-->

## What was not done

<!--
Anything skipped, unmeasured or unverified, including a test skipped for a
resource this machine does not have. A negative disclosure here stays negative.
-->
