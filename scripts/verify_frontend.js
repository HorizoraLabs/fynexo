const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

function walkFiles(dir, exts) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fp = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...walkFiles(fp, exts));
    } else if (exts.test(entry.name)) {
      out.push(fp);
    }
  }
  return out;
}

const jsTargets = [
  ...walkFiles(path.join('src', 'hud'), /\.js$/i),
  ...walkFiles(path.join('src', 'pages'), /\.js$/i),
].sort();

for (const jsFile of jsTargets) {
  execFileSync(process.execPath, ['--check', jsFile], { stdio: 'inherit' });
}

const allSrcFiles = walkFiles('src', /\.(js|html|css|rs)$/i);
const emojiRegex = /[\u{1F000}-\u{1FAFF}\u2600-\u27BF\u231A-\u23FF\u2B50\uFE0F\u20E3]/gu;

const emojiHits = [];
const innerHtmlHits = [];
const dupIdHits = [];
const dynamicEvalHits = [];

for (const file of allSrcFiles) {
  const norm = file.replace(/\\/g, '/');
  const txt = fs.readFileSync(file, 'utf8');
  const lines = txt.split(/\r?\n/);

  lines.forEach((line, idx) => {
    const emojiMatches = line.match(emojiRegex);
    if (emojiMatches) {
      emojiHits.push(`${norm}:${idx + 1} -> ${JSON.stringify(emojiMatches)}`);
    }

    if (norm.endsWith('.js') && (norm.startsWith('src/hud/') || norm.startsWith('src/pages/'))) {
      if (/\.innerHTML\b/.test(line)) {
        innerHtmlHits.push(`${norm}:${idx + 1} -> ${line.trim()}`);
      }

      if (/\beval\s*\(/.test(line)) {
        const ctx = lines.slice(Math.max(0, idx - 24), idx + 3).join('\n');
        const isAuditedDevRepl =
          norm === 'src/hud/runtime.js' &&
          line.includes('window.eval(code)') &&
          ctx.includes('config.allow_page_eval');
        if (!isAuditedDevRepl) {
          dynamicEvalHits.push(`${norm}:${idx + 1} (unaudited eval) -> ${line.trim()}`);
        }
      }

      if (/\bnew\s+Function\s*\(/.test(line)) {
        const ctx = lines.slice(Math.max(0, idx - 6), idx + 4).join('\n');
        const isAuditedScriptForge =
          norm === 'src/hud/runtime.js' &&
          ctx.includes('createScriptForgeSandbox');
        if (!isAuditedScriptForge) {
          dynamicEvalHits.push(`${norm}:${idx + 1} (unaudited new Function) -> ${line.trim()}`);
        }
      }
    }
  });

  if (norm === 'src/pages/settings.html' || norm === 'src/pages/newtab.html') {
    const seen = new Map();
    for (const m of txt.matchAll(/\bid\s*=\s*"([^"]+)"/g)) {
      const id = m[1];
      seen.set(id, (seen.get(id) || 0) + 1);
    }
    for (const [id, count] of seen.entries()) {
      if (count > 1) {
        dupIdHits.push(`${norm}: id="${id}" appears ${count} times`);
      }
    }
  }
}

console.log('SYNTAX_CHECKED_JS=' + jsTargets.length);
console.log('SCANNED_FILES=' + allSrcFiles.length);
console.log('EMOJI_COUNT=' + emojiHits.length);
emojiHits.forEach((h) => console.error('EMOJI:', h));
console.log('INNERHTML_COUNT=' + innerHtmlHits.length);
innerHtmlHits.forEach((h) => console.error('INNERHTML:', h));
console.log('DUP_ID_COUNT=' + dupIdHits.length);
dupIdHits.forEach((h) => console.error('DUP_ID:', h));
console.log('UNAUDITED_DYNAMIC_CODE=' + dynamicEvalHits.length);
dynamicEvalHits.forEach((h) => console.error('UNAUDITED_EVAL:', h));

if (emojiHits.length || innerHtmlHits.length || dupIdHits.length || dynamicEvalHits.length) {
  process.exit(1);
}
