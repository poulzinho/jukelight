## 1. Stack-layer rule (one layer: a single pull request, no stack needed)

- [x] 1.1 Check first that the rule is missing: `openspec instructions tasks --change dev-2-add-the-stack-layer-rule-to-the-openspec-tasks-rules --json` lists no rule mentioning stack layers
- [x] 1.2 Add the rule to `rules.tasks` in `openspec/config.yaml` (each task group is one stack layer: one branch, one pull request, one Linear sub-issue of the change's issue; it builds and passes CI on its own; layers ordered bottom-up); verify `openspec doctor` reports no parse problem and the command from 1.1 now lists the rule
- [ ] 1.3 Verify nothing else changed: `git diff --stat` shows only `openspec/config.yaml` and this change's artifacts, and CI is green on the pull request
