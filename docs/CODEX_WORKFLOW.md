# Efficient Codex Workflow

Use one fresh Codex task for one mode and one atomic requirement or PR repair.
Start every task from the GeoRBF local project so it sees the same working tree,
but do not use one conversation as project history. Durable state lives in
`requirements/v1.yaml`, `docs/progress/CURRENT.md`, Git, and GitHub.

## What the repository configures

- `.codex/config.toml` uses `high` reasoning for normal GeoRBF work and limits
  agent fan-out to four threads at one level.
- `.codex/agents/math_reviewer.toml` is a read-only `xhigh` independent
  reviewer for mathematical and numerical PRs.
- Draft PRs run the complete correctness gate on Ubuntu without benchmark
  smoke. Ready PRs and `main` run the full Windows, Ubuntu, and macOS matrix
  with every benchmark smoke workload.
- `CURRENT.md` stays bounded. Historical evidence is indexed by
  `docs/progress/HISTORY.md` instead of being copied into every task.

## Compact requirement commands

Run these instead of printing the whole requirement registry:

```text
cargo xtask requirements next
cargo xtask requirements show REQ-POLY-001
cargo xtask requirements deps REQ-POLY-001
```

`next` prefers unfinished active work, then the earliest eligible milestone,
priority, and registry order. `show` returns the requirement, dependency
statuses, tests, documents, interfaces, Issue, and PR in a compact summary.

## Prompt for a new task

Use this when opening a new task and you want Codex to select the correct next
mode from repository state:

```text
这是一个新的 GeoRBF 任务。先读取 AGENTS.md 和
docs/progress/CURRENT.md，执行强制预检并检查远端 Issue、PR、Review 和
CI。按 Repair > Review > Implement > Release 的优先级，只选择一个模式，
并且只处理一个原子 requirement 或一个 PR repair。

使用 cargo xtask requirements next/show/deps 获取精简上下文；不要读取完整
requirements/v1.yaml，除非正在修改 validator 或执行 Release。开发中只跑
focused checks；最后一次代码修改后，在稳定 head 上运行一次完整标准检查。
达到该模式退出条件后更新 bounded handoff、提交、推送并停止。不要继续下一个
requirement。
```

## Prompt after Implement finishes

Open a new task. Do not send this as a follow-up in the implementation task:

```text
这是一个新的 GeoRBF Review 任务。只审查 PR #<PR号> 和对应的单一
requirement；不要修复生产代码，也不要开始下一个 requirement。

先执行强制预检，然后显式创建并等待项目 math_reviewer 子代理。只向审查代理
提供 requirement show/deps 摘要、相关规范与 ADR、PR diff、测试和 benchmark
证据，不继承实现任务的推理历史。要求按 P0-P3 输出带文件和行号的发现、独立
真值推理及所需回归测试。父任务把审查结论记录到 PR/Review 文档后停止。
```

## Prompt when Review finds defects

Open another new task:

```text
这是一个新的 GeoRBF Repair 任务。只修复 PR #<PR号> 中已确认的
<发现编号或审查线程>，不要扩大 requirement，也不要开始其他工作。

先复现问题并添加独立回归测试，再实施最小修复。迭代时运行 focused checks；
最后一次代码修改后运行完整标准检查，更新 review evidence 和 bounded handoff，
提交并推送，然后停止，等待新的独立 re-review 任务。
```

## Prompt for re-review and integration

Open a fresh task after repairs, or immediately after a no-finding first review:
The mandatory integration sequence is: mark the PR ready -> wait for the
complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact ready head ->
merge exactly once only when that CI is green -> record truthful integration
state.

```text
这是一个新的 GeoRBF Review/re-review 任务。只处理 PR #<PR号>。
执行强制预检，显式创建项目 math_reviewer 子代理，独立确认原始发现是否关闭并
检查是否出现新问题。若仍有发现，记录证据后停止；不要自行修复。

若无 P0-P3 发现且最终 head 的完整本地检查为绿色，则同步 PR 证据并标记 ready；
等待由 ready 事件触发的 Windows、Ubuntu、macOS 和全部 benchmark smoke CI。
只有该完整 CI 为绿色时才按仓库策略合并，并通过隔离的 integration-state 变更
如实更新 registry 与 bounded handoff。完成后停止，不要开始下一个
requirement。
```

## Prompt for the next requirement

Only after the preceding requirement is truthfully `integrated`, open a new
task with the general new-task prompt above. A shorter equivalent is:

```text
这是一个新的 GeoRBF 任务。执行 AGENTS.md 的强制预检，确认没有更高优先级的
Repair 或 Review 后，只实施 cargo xtask requirements next 返回的一个原子
requirement。到 Draft PR 和稳定 head 的完整本地 gate 为止，然后停止；独立
Review 必须在新任务中进行。
```

## Prompt for a safe resume

If a task stops at a committed boundary because of context growth or an
external interruption, resume in a new task:

```text
这是一个新的 GeoRBF 续作任务。以 docs/progress/CURRENT.md、当前分支和 PR
状态为准，不依赖旧聊天。执行强制预检，只恢复 handoff 指定的同一 mode、同一
requirement 或同一 repair。核对已有提交和已通过检查，避免重复读取历史或重复
运行未变化 head 的完整 gate；完成当前模式后停止。
```

## Operational guardrails

- Never type “continue the whole project” inside an existing long task.
- Start a new task at every Implement, Review, Repair, and next-requirement
  boundary.
- If a task requires a second context compaction, commit or preserve a safe
  boundary, update `CURRENT.md`, and resume fresh.
- Do not run two write-capable tasks against the same checkout. Use a read-only
  reviewer or separate worktree for genuine parallel work.
- Fast mode is optional and is not a substitute for bounded context.
