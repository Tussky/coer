---
name: next
description: Ask the advisor what to work on next. Reads the working tree, recent commits, open pull requests, outstanding review findings, and TODOs, then recommends one concrete next step. Use when unsure how to proceed, or when a change has landed and it is unclear what follows.
allowed-tools: Read, Grep, Glob, Bash, Agent
---

# What next?

Delegate to the **`next-step-advisor`** agent and relay its recommendation.

Pass along anything the human said after `/next` as their stated intent — `/next I want to get this ready to show someone` and `/next` alone should produce different advice. If they said nothing, tell the advisor there is no stated goal and it should infer one from repository state, or ask.

Brief the agent with:

> The human is asking what to work on next. Stated intent: `<their words, or "none given">`.
> Repository root: `<path>`. Current branch: `<branch>`.
> Inspect repository state yourself — working tree, recent commits, open pull requests and their review status, recent TODO/FIXME markers, `docs/ARCHITECTURE.md`, and any recent review panel comment. Recommend exactly one next step. Do not modify anything.

Relay the advisor's answer without expanding it. Do not append your own alternative plan, and do not start doing the work — the human decides whether to act.

If they then ask you to carry out the recommendation, treat that as a fresh, explicit instruction and do it normally.

## Related

- `/panel` — review the current diff before committing.
- `/wrapup` — sync documentation and the Graphify graph after a change lands.
- To consult one specialist directly, name it: *"use the security-reviewer agent on `src/auth.rs`"*.
