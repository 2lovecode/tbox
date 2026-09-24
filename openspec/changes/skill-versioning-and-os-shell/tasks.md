# Tasks

## 1. Registry And OS Shell Backend

- [x] 1.1 Extend `SideEffect` with `Process` and register `os.shell` in `registry.rs`; verify `lookup("os.shell")` and schema reject missing `command`
- [x] 1.2 Implement restricted shell executor (timeout, truncate, denylist, cwd default/override, command probe); verify unit tests for denylist and safe command
- [x] 1.3 Wire `os.shell` dispatch to executor; verify `cargo test` for registry/os_shell modules pass

## 2. Approval Gate

- [x] 2.1 Add `ToolApprovalRequired` event and pending approval map with `resolve_tool_approval` command; verify interrupt denies pending
- [x] 2.2 Gate `Process` tools in agent loop (allow / allow_similar / deny); verify unit/integration test for session similarKey
- [x] 2.3 Frontend approval bar on chat (`HomePage` or composable) with three actions; verify UI invokes resolve command

## 3. Skill Versioning

- [x] 3.1 Implement override + versions storage and resolve path in `skills.rs`; verify override wins over embedded
- [x] 3.2 Extend `update_skill` for builtin; add `list_skill_versions` / `restore_skill_version` / `restore_skill_default`; verify restore flows in tests
- [x] 3.3 Update `SkillSettingsPanel` for edit builtin, modified badge, history UI; verify list shows new fields

## 4. Skills Content And Settings

- [x] 4.1 Rewrite all 16 builtin skill markdown files to standard structure; verify frontmatter + ≥2 samples each
- [x] 4.2 Add `os.shell.md` skill and register in `SKILLS`; verify coverage check includes `os.shell`
- [x] 4.3 Add Shell default cwd setting (prefs + UI); verify `os.shell` uses it when cwd omitted

## 5. Tests And Eval

- [x] 5.1 Update tool-count / skill-coverage / eval assertions for 18 tools / 17 skills; verify `cargo test` relevant suites pass
- [x] 5.2 Mark OpenSpec tasks complete and run `openspec validate --change skill-versioning-and-os-shell`
