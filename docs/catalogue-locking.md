# One writer per catalogue

Issue: #35

Two instances writing one catalogue is a corruption bug that presents as a
mystery. The damage is done by the second writer and found by whoever opens the
catalogue next, which may be weeks later and is never the person who caused it.
This note says what `crates/catalogue/src/lock.rs` does about that, and it says
the two things it deliberately does not do, because both are restrictions an
operator can meet and neither is visible from the code that refuses.

## What holds the catalogue

One file inside the catalogue directory, `catalogue.lock`, and the operating
system's advisory lock on it, taken through `std::fs::File::try_lock`. The lock
belongs to the open file rather than to the path, so it is released when the
file is closed and it is released when the process holding it ends, however it
ends.

Nothing writes a lock this module later has to judge the freshness of. Nothing
reads a process identifier to decide whether a holder is still alive. That is
the whole of the answer to the case which produces most of the reports about
locking elsewhere, a previous instance that died without releasing anything: a
stale lock is not a state this module recovers from, it is a state that cannot
arise, and no operator ever deletes a file by hand to get back in. What proves
it is `crates/catalogue/tests/single_writer.rs`, which kills a holder rather
than asking it to exit and then opens the catalogue.

A second file, `catalogue.lock.holder`, carries what the holder can say about
itself, so a refused process can name it. It is a separate file rather than the
body of the locked one because an exclusive lock on Windows refuses another
process's read of the locked range, so a holder writing its description into
the file it locked would have put it where the only reader who needs it cannot
reach it.

Everything in that description is what the holder said about itself. None of it
is checked against the operating system. It names a holder for a person to act
on and it proves nothing about one, and a description that is missing, half
written or unparseable produces a refusal that says a process is holding the
catalogue and that it could not be named. That is a different sentence from
naming one, and the two are kept apart so the second cannot be printed when
only the first is true.

## Reading while another process writes is refused

A read-only open takes a shared lock on the same file, so it succeeds while
other readers hold it and is refused while a writer does. That is a real
restriction rather than an oversight, and it is here rather than worked around
in the module.

Admitting a reader alongside a writer would be a claim about what the storage
engine underneath does to its files mid-write. Which engine that is has not been
chosen: issue #5 measures the candidates and issue #6 chooses one. A promise
made now would be a promise about an engine nobody has picked, and the honest
version of it is this paragraph.

The second restriction is smaller and follows from the first. A reader opens the
lock file for writing and creates it when it is absent, so opening a catalogue
on a volume mounted read-only fails at the lock rather than at the catalogue.
The refusal says the lock could not be taken and that nothing was opened, which
is true but does not name the volume as the reason.

## On a network share the promise is not established

The lock is the operating system's, so it is exactly as strong as the filesystem
holding the file. A file server that answers a lock request without honouring it
admits two writers and tells neither, and that is the one case where the whole
of the above fails silently.

So an open says what it established about where it was taken rather than leaving
a caller to assume. It answers one of two things. `Location::Network` means the
path was recognised as naming another machine's filesystem, so the single-writer
promise rests on that server rather than on this kernel and was not established
here. `Location::NotRecognisedAsNetwork` means the path was not recognised as a
network location.

The second answer is not the statement that the path is local, and it is spelled
the way it is so that it cannot be read as one. What is recognised exactly is a
Windows UNC path, in both separators and in the verbatim form, because a UNC
path names another machine in its own spelling. A mapped drive letter, a Unix
mount of a remote filesystem and a bind mount over one all reach the second
answer. An operator reading it learns that nothing was found, not that nothing
is there.

The value carries `#[must_use]` on the functions that produce it, which is what
makes the statement reach a caller. A location nobody is obliged to bind is a
location nobody has to read.

## What is enforced and what is asked for

Enforced. The refusal itself, the recovery from a holder that was killed, and
the single winner among processes opening at once are the three tests in
`crates/catalogue/tests/single_writer.rs`. The classification of a path and the
parsing of a holder description are unit tests in the module.

Asked for. That a caller prints the location it was handed. Nothing in this tree
reads the surface's output, and `#[must_use]` refuses only a discarded value,
not a bound one that is then ignored. Issue #118 is where the first run's
messages are decided, and that is where this stops being a convention.

Not established anywhere. Whether a given filesystem honours the lock it
accepted. No test here can settle it, because the answer belongs to the file
server rather than to this tree, and the module says so at the open instead of
asserting it.
