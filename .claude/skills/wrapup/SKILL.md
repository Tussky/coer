---
name: wrapup
description: After a change lands, bring documentation and the Graphify codebase graph back in sync with the code. Opens a pull request with the updates rather than pushing. Use after merging a pull request, or when docs feel stale.
allowed-tools: Read, Grep, Glob, Bash, Agent
---

# Wrap up after a merge

Delegate to the **`docs-updater`** agent.

Work out what landed before dispatching, so the agent gets a precise range rather than guessing:

- **`--base` and `--head` were both passed** — use them. This is how CI calls you after a pull
  request merges.
- **Only one, or neither, was passed** — this is a manual run (`workflow_dispatch` supplies no
  pull request, so the `--base` placeholder expands to nothing). Fall back to the last merge:
  `git diff HEAD~1 HEAD`, or `git log --merges -1` to find the most recent merge commit and diff
  against its first parent.
- **No commits at all** — the repository's first commit has not happened. Nothing to sync; say so
  and stop.
- **Locally** — the most recent merge, or whatever range the human named after `/wrapup`.

Brief the agent with:

> A change has landed. Range: `<range>`. Repository root: `<path>`.
> Read the diff, update only the documentation the change makes inaccurate, refresh the Graphify graph with `graphify . --update`, and open a pull request with the result.
> Do not edit source code. Do not push to the default branch. Do not merge anything.
> If nothing is stale, open no pull request and say so.

## Guardrails

These are the properties that make this safe to run automatically. Do not let them slip:

- **Documentation only.** Source, tests, CI, and dependency manifests are off limits. If the agent reports that a code change is needed, relay that to the human as a note — do not act on it here.
- **Pull request, never a direct push.** The human reviews and merges every documentation change, exactly as they would their own.
- **`[skip-agents]` in the commit message.** Without it the post-merge workflow retriggers on its own pull request and loops.
- **Graphify failure is not run failure.** If `graphify` is missing or errors, the documentation updates still go ahead. Report the failure in one line.

## Report back

Relay, in three lines at most: the pull request URL or `no documentation changes needed`; whether the Graphify graph refreshed; and anything the agent explicitly flagged for the human's judgement. Do not restate the full pull request body — they can read it.

## Related

- `/panel` — review a diff before committing.
- `/next` — decide what to work on next.
