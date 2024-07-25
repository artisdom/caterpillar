# Design

## About

**Please note: This document is on its way to being deprecated.**

- Caterpillar is an early-stage project, and the plans for its design are still
  speculative; a work in progress. Yet, this document gives the impression that
  they are stable, my warning in the introduction notwithstanding. I don't like
  that.
- I've started documenting design decisions that have already been implemented
  in the [README](README.md).
- And I've started talking about the more transient design ideas in a format
  that better fits their nature, my
  [daily thoughts](https://capi.hannobraun.com/daily).
- Over time, I'll remove more and more topics from this document, as they are
  handled in either (or both) of those places.

---

Caterpillar has always been developed as a series of prototypes. Each of those
prototypes had a limited scope, exploring specific objectives. This document
attempts to explain the long-term vision I've been working towards, separate
from any specific prototype.

Please note that while I have been a student of programming for most of my life,
I am not an experienced language designer. These ideas present a snapshot of my
current thinking, and are sure to change as they come into contact with reality.

Please also note that this document is currently incomplete and a work in
progress. I intend to keep working on it, adding new concepts and keeping the
existing ones up-to-date.

## Concepts

**Please note: I've started removing topics here, as per the note above.**

Some of the remaining ones might still reference removed ones. I don't intend to
update them to fix that, as the rest of them are also due to be removed sooner
or later.

---

### Content-addressed definitions

Definitions in Caterpillar, functions, types, etc, will be content-addressed,
meaning they are identified by a hash of their contents.

This means that multiple versions of the same definition can exist and be
referred to at the same time. It also implies that the canonical form of code is
stored in a form that is not the same as the textual representation that a
programmer would write.

This idea is lifted from [Unison]. I won't go into justification here, as
Unison's documentation already does a great job of explaining the benefits. I
would like to expand on some points though, that I haven't seen addressed on
their side.

The straight-forward way to implement this, is to store code in some kind of
structured database. I think, but I'm not sure, that that's how Unison does
that. This has the disadvantage of either being tedious to use, or requiring
specialized tooling, or likely both.

I have come up with an alternative way: The written form of Caterpillar form
lives in regular text files, meaning no special tooling is required to edit it.
Since Caterpillar is interactive, it needs to constantly monitor those files for
changes anyway, to apply changes to the running program. When it processes these
files, it can create, update, and take into account a second set of files, which
contains the canonical representation.

Here's an example, to hopefully make that understandable:

1. The programmer writes code that calls a function: `x`
2. Caterpillar sees that no canonical representation of that code exists yet,
   and will now create it.
3. Caterpillar resolves this function call to the function `x` with hash `1`.
4. Caterpillar writes the canonical representation of the new code `x@1`.
5. A new version of function `x` with the hash `2` is defined.
6. The programmer makes changes to the original code; since the canonical
   representation exists, Caterpillar knows that the original mention of `x`
   still refers to `x@1`.
7. New mentions of `x` will resolve to `x@2`. This distinction will be displayed
   by tooling in a way that preserves clarity.
8. The programmer can upgrade the original mentions of `x` to refer to `x@2`,
   through some kind of interface (could be CLI; GUI, integrated into the IDE,
   ...). This upgrade could possibly be automatic, if `x@1` and `x@2` have
   compatible signatures.

Both the written and the canonical representation would live side-by-side in
version control.

[Unison]: https://www.unison-lang.org/

## Future Extensions

This design is, as stated in the introduction, not complete. Besides accidental
omissions, I'm actively thinking about the following topics:

- [Interaction nets]: Those could be a better basis for computation, but I need
  to study them more. This is also inspired by HVM.
- Homoiconicity: Seems like a desirable property, but it's not something I have
  thought about much, so far.

[Interaction nets]: https://en.wikipedia.org/wiki/Interaction_nets
