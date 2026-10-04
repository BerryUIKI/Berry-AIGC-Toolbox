import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

test('tauri.conf.json: NSIS installer is configured with Start Menu folder, languages, and hooks', () => {
  const tauriConfPath = path.join(projectRoot, 'src-tauri', 'tauri.conf.json');
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));

  const nsis = tauriConf.bundle?.windows?.nsis;
  assert.ok(nsis, 'NSIS configuration must exist');
  assert.equal(nsis.startMenuFolder, 'Omera', 'Start Menu folder must be Omera');
  assert.equal(nsis.installMode, 'currentUser', 'Install mode must be currentUser');
  assert.equal(nsis.installerHooks, 'windows/hooks.nsh', 'installerHooks must be configured');

  // Verify hooks file exists on disk
  const hooksPath = path.join(projectRoot, 'src-tauri', nsis.installerHooks);
  assert.ok(fs.existsSync(hooksPath), `hooks.nsh must exist at ${hooksPath}`);

  // Verify languages array includes all 7 supported locales
  const requiredLanguages = ['English', 'SimpChinese', 'TradChinese', 'Japanese', 'German', 'French', 'Spanish'];
  assert.ok(Array.isArray(nsis.languages), 'languages must be an array');
  for (const lang of requiredLanguages) {
    assert.ok(nsis.languages.includes(lang), `NSIS must include language: ${lang}`);
  }
});

test('i18n locales: all 7 locale files define desktop shortcut keys and sourceModelScope', async () => {
  const locales = ['zh-CN', 'zh-TW', 'en', 'ja', 'de', 'fr', 'es'];
  for (const locale of locales) {
    const localeFile = path.join(projectRoot, 'src', 'i18n', 'locales', `${locale}.ts`);
    const content = fs.readFileSync(localeFile, 'utf8');
    assert.ok(content.includes('desktopShortcut:'), `${locale}.ts must define desktopShortcut`);
    assert.ok(content.includes('createDesktopShortcut:'), `${locale}.ts must define createDesktopShortcut`);
    assert.ok(content.includes('desktopShortcutCreated:'), `${locale}.ts must define desktopShortcutCreated`);
    assert.ok(content.includes('sourceModelScope:'), `${locale}.ts must define sourceModelScope`);
  }
});

test('commands.rs: contains ModelScope repository URL for BerryUIKI tagger', () => {
  const commandsFile = path.join(projectRoot, 'src-tauri', 'src', 'commands.rs');
  const content = fs.readFileSync(commandsFile, 'utf8');
  assert.ok(
    content.includes('https://modelscope.cn/models/BerryUIKI/'),
    'commands.rs must contain ModelScope repository URLs for BerryUIKI'
  );
});

