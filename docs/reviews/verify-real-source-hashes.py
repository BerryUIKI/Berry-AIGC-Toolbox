import hashlib, json, pathlib
review = pathlib.Path(r'D:\dev\Omera\docs\reviews')
manifest = json.loads((review / 'real-test-fixtures-2026-10-05.json').read_text(encoding='utf-8-sig'))
results = [{'alias': row['alias'], 'source_unchanged': hashlib.sha256(pathlib.Path(row['path']).read_bytes()).hexdigest() == row['source_sha256'], 'fixture_unchanged': hashlib.sha256(pathlib.Path(row['fixture']).read_bytes()).hexdigest() == row['source_sha256']} for row in manifest['fixtures']]
summary = {'checked': len(results), 'source_unchanged': sum(r['source_unchanged'] for r in results), 'fixture_unchanged': sum(r['fixture_unchanged'] for r in results)}
(review / 'real-source-hash-verification-2026-10-05.json').write_text(json.dumps({'summary': summary, 'fixtures': results}, indent=2), encoding='utf-8')
print(json.dumps(summary))
if not all(r['source_unchanged'] and r['fixture_unchanged'] for r in results): raise SystemExit(1)
