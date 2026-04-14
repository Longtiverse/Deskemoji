# Emoji State Review Board Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone HTML emoji state review board that shows all eight Deskemoji states at once with real asset-driven previews, persistent reviewer notes, and export tools.

**Architecture:** Keep the runtime surface as a single HTML file under `docs/review/`, but drive it from one structured in-page state configuration so preview behavior, trigger copy, asset paths, and export data stay in sync. Add one lightweight Node test file that verifies the board exists and includes the required review, persistence, and export affordances before and after implementation.

**Tech Stack:** Standalone HTML/CSS/vanilla JavaScript, Node built-in test runner, repo-local PNG assets

---

### Task 1: Lock review-board requirements with a failing test

**Files:**
- Create: `D:\Project\Deskemoji\tests\review_board_html_test.mjs`
- Test: `D:\Project\Deskemoji\tests\review_board_html_test.mjs`

- [ ] **Step 1: Write the failing test**

```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

test('review board html exists and includes required review affordances', () => {
  const html = fs.readFileSync('docs/review/deskemoji-emoji-state-review-board.html', 'utf8');
  assert.match(html, /const REVIEW_STATES = \[/);
  assert.match(html, /localStorage/);
  assert.match(html, /导出 JSON/);
  assert.match(html, /导出 Markdown/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/review_board_html_test.mjs`
Expected: FAIL because `docs/review/deskemoji-emoji-state-review-board.html` does not exist yet

- [ ] **Step 3: Expand the failing test to cover the approved state set**

```javascript
for (const stateId of ['happy', 'sad', 'angry', 'sleepy', 'thinking', 'hot', 'mindblown', 'goodnight']) {
  assert.match(html, new RegExp(`id:\\s*'${stateId}'`));
}
```

- [ ] **Step 4: Run test to verify it still fails for the expected reason**

Run: `node --test tests/review_board_html_test.mjs`
Expected: FAIL because the target HTML file is still missing

### Task 2: Build the standalone review board

**Files:**
- Modify: `D:\Project\Deskemoji\.gitignore`
- Create: `D:\Project\Deskemoji\docs\review\deskemoji-emoji-state-review-board.html`
- Test: `D:\Project\Deskemoji\tests\review_board_html_test.mjs`

- [ ] **Step 1: Add the generated brainstorming workspace to gitignore without disturbing existing entries**

```gitignore
.superpowers/
```

- [ ] **Step 2: Create a structured review-state configuration inside the HTML**

```javascript
const REVIEW_STATES = [
  {
    id: 'happy',
    number: '01',
    labelZh: '开心',
    emojiChar: '🙂',
    stem: 'happy',
    autoMode: true,
    triggerSummary: '默认回退状态',
  },
];
```

- [ ] **Step 3: Build the board shell and top summary**

```html
<header class="hero">
  <h1>Deskemoji Emoji 状态说明与评审看板</h1>
  <div class="rule-strip">Hot > Mindblown > Sleepy > Goodnight > Thinking > Happy</div>
</header>
```

- [ ] **Step 4: Render one review card per state**

```javascript
board.innerHTML = REVIEW_STATES.map(renderStateCard).join('');
```

- [ ] **Step 5: Implement real asset-driven preview behavior**

```javascript
function assetPath(stem, suffix = '') {
  return `../../assets/emoji/${stem}${suffix ? `-${suffix}` : ''}.png`;
}
```

- [ ] **Step 6: Port the preview math needed for idle motion, gaze direction, and closed-eye response**

```javascript
function chooseDirection(dx, dy) {
  const horizontal = dx <= -36 ? -1 : dx >= 36 ? 1 : 0;
  const vertical = dy <= -42 ? -1 : dy >= 48 ? 1 : 0;
  return `${vertical},${horizontal}`;
}
```

- [ ] **Step 7: Add state note inputs, status chips, auto-save, and restore**

```javascript
localStorage.setItem(STORAGE_KEY, JSON.stringify(reviewState));
```

- [ ] **Step 8: Add JSON and Markdown export actions**

```javascript
downloadExport('deskemoji-review.json', JSON.stringify(payload, null, 2), 'application/json');
downloadExport('deskemoji-review.md', buildMarkdown(payload), 'text/markdown');
```

- [ ] **Step 9: Run the Node test to verify the new board passes**

Run: `node --test tests/review_board_html_test.mjs`
Expected: PASS

### Task 3: Verify the board is a usable development artifact

**Files:**
- Modify: `D:\Project\Deskemoji\docs\review\deskemoji-emoji-state-review-board.html`
- Test: `D:\Project\Deskemoji\tests\review_board_html_test.mjs`

- [ ] **Step 1: Re-read the spec and compare the HTML against the required card contents**

Checklist:
- all eight states visible at once
- real asset paths used
- hover/follow preview present
- closed-eye states avoid fake pupils
- local persistence present
- JSON and Markdown export present

- [ ] **Step 2: Run the focused test again after any cleanup**

Run: `node --test tests/review_board_html_test.mjs`
Expected: PASS

- [ ] **Step 3: Record manual verification notes in the final response**

Manual checks:
- open the HTML directly in a browser
- move the pointer over several cards and confirm frame-follow or closed-eye response
- type notes, refresh, and confirm they restore
- export both formats and inspect filenames/content
