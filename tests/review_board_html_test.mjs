import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const htmlPath = path.resolve('docs/review/deskemoji-emoji-state-review-board.html');

function readHtml() {
  return fs.readFileSync(htmlPath, 'utf8');
}

test('review board html exists and includes required review affordances', () => {
  const html = readHtml();

  assert.match(html, /const REVIEW_STATES = \[/);
  assert.match(html, /localStorage/);
  assert.match(html, /导出 JSON/);
  assert.match(html, /导出 Markdown/);
  assert.match(html, /Deskemoji Emoji 状态说明与评审看板/);
});

test('review board declares all approved emoji states and review inputs', () => {
  const html = readHtml();
  const stateIds = [
    'happy',
    'sad',
    'angry',
    'sleepy',
    'thinking',
    'hot',
    'mindblown',
    'goodnight',
  ];

  for (const stateId of stateIds) {
    assert.match(html, new RegExp(`id:\\s*'${stateId}'`));
    assert.match(html, new RegExp(`feedback-${stateId}`));
  }

  assert.match(html, /保留/);
  assert.match(html, /待改/);
  assert.match(html, /问题大/);
});
