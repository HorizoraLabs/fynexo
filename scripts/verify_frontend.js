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
          (norm === 'src/hud/runtime.js' || norm === 'src/hud/content_agent.js') &&
          line.includes('window.eval(code)') &&
          ctx.includes('config.allow_page_eval');
        if (!isAuditedDevRepl) {
          dynamicEvalHits.push(`${norm}:${idx + 1} (unaudited eval) -> ${line.trim()}`);
        }
      }

      if (/\bnew\s+Function\s*\(/.test(line)) {
        const ctx = lines.slice(Math.max(0, idx - 6), idx + 4).join('\n');
        const isAuditedScriptForge =
          (norm === 'src/hud/runtime.js' || norm === 'src/hud/content_agent.js') &&
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

const requiredTokensFile = path.join('src', 'hud', 'tokens.css');
if (!fs.existsSync(requiredTokensFile)) {
  console.error('MISSING_TOKENS_CSS:', requiredTokensFile);
  process.exit(1);
}
const allowedScalePx = new Set(['0', '0px', '2px', '3px', '4px', '5px', '6px', '7px', '8px', '9px', '10px', '10.5px', '11px', '11.5px', '12px', '12.5px', '13px', '13.5px', '14px', '14.5px', '15px', '16px', '18px', '20px', '42px', '44px', '50%', '999px']);
const cssTokenViolations = [];
for (const cssFile of [path.join('src', 'hud', 'tokens.css')]) {
  const txt = fs.readFileSync(cssFile, 'utf8');
  for (const m of txt.matchAll(/(?:border-radius|font-size)\s*:\s*([^;]+);/g)) {
    const val = m[1].trim();
    if (!val.startsWith('var(') && !allowedScalePx.has(val)) {
      cssTokenViolations.push(`${cssFile}: off-scale value ${val}`);
    }
  }
}

const themeDriftViolations = [];
const leakedDefaultHexRegex = /#(?:0c0e13|0d0f14|1c212d|151923)\b/i;
for (const jsFile of jsTargets) {
  const norm = jsFile.replace(/\\/g, '/');
  const txt = fs.readFileSync(jsFile, 'utf8');
  if ((norm === 'src/pages/settings.js' || norm === 'src/pages/newtab.js') && /\bisDefaultDark\b/.test(txt)) {
    themeDriftViolations.push(`${norm}: forbidden isDefaultDark hardcoded branch`);
  }
  const lines = txt.split(/\r?\n/);
  lines.forEach((line, idx) => {
    if (leakedDefaultHexRegex.test(line)) {
      themeDriftViolations.push(`${norm}:${idx + 1}: hardcoded default Obsidian hex (${line.trim()})`);
    }
  });
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
console.log('TOKEN_VIOLATIONS=' + cssTokenViolations.length);
cssTokenViolations.forEach((v) => console.error('TOKEN_VIOLATION:', v));
console.log('THEME_DRIFT_VIOLATIONS=' + themeDriftViolations.length);
themeDriftViolations.forEach((v) => console.error('THEME_DRIFT:', v));

if (emojiHits.length || innerHtmlHits.length || dupIdHits.length || dynamicEvalHits.length || cssTokenViolations.length || themeDriftViolations.length) {
  process.exit(1);
}
