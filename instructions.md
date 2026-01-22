## Agent Instructions
---
1. Read activity.md first to understand current state
2. Select the next task where passes: false (only one)
3. Implement only that task.
4. Run verification commands: `cargo check` and fix until they pass.
5. Update only that task’s passes field to true (do not edit task text/ordering).
6. Append a short log line to activity.md (timestamp + task id + what was verified).
9. If all tasks have passes: true, output exactly <completion-text> and nothing else.

Important: Only modify the passes field. Do not remove or rewrite tasks.
---

## Completion Criteria
---
All tasks marked with "passes": true
```md
### Step 4: Create activity.md

This file logs what the agent accomplishes during each iteration:

```markdown
# Project Build - Activity Log

## Current Status
**Last Updated:** 
**Tasks Completed:** 
**Current Task:** 

---

## Session Log

<!-- Agent will append dated entries here -->
```
---
