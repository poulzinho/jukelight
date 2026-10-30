# DEV-2: Add the stack-layer rule to the OpenSpec tasks rules

Linear issue: DEV-2 (chore). Not a roadmap step: it changes how work is
planned, not what Jukelight does.

## Why

DEV-1 was delivered as one large pull request, which made review slow.
From now on, larger changes are delivered as stacked pull requests with
`gh stack`, and each pull request gets its own Linear sub-issue. The task
list of a change decides how the stack is cut, so the rule belongs in the
OpenSpec tasks rules, where every future `tasks.md` is planned.

## What Changes

- Add a rule to `rules.tasks` in `openspec/config.yaml`: group tasks so each
  group becomes one layer of the stack, that is one branch, one pull request
  and one Linear sub-issue of the change's issue. Each layer builds and
  passes CI on its own, and layers are ordered bottom-up (for example domain,
  then adapters, then API, then kiosk, then delivery).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is workflow configuration; no product behaviour changes, so the
change declares `skip_specs: true`.

## Non-goals

- Restructuring already merged work (DEV-1 stays one pull request).
- Automating sub-issue creation or stack submission.
- Changing other artifact rules or the project context.

## Impact

- `openspec/config.yaml`: one new tasks rule. OpenSpec shows it in the tasks
  instructions of every new change.
- No code, dependencies or APIs change.
