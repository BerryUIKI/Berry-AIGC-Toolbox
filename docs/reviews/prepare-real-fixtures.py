import collections, hashlib, json, pathlib, shutil, subprocess, tempfile
from PIL import Image

root = pathlib.Path(r"D:\AIGC")
review = pathlib.Path(r"D:\dev\Omera\docs\reviews")
listed = subprocess.run(["rg", "--files", "--hidden", "--no-ignore", str(root)], capture_output=True, text=True, encoding="utf-8", check=True).stdout.splitlines()
rows = []
errors = []
for name in listed:
    path = pathlib.Path(name)
    if path.suffix.lower() not in {".png", ".jpg", ".jpeg", ".webp", ".avif"}:
        continue
    try:
        with Image.open(path) as im:
            keys = sorted(im.info.keys())
            category = "A1111" if "parameters" in keys else "ComfyUI" if "prompt" in keys or "workflow" in keys else "Other/none"
            rows.append({"path": str(path), "folder": str(path.relative_to(root).parts[0]), "bytes": path.stat().st_size, "width": im.width, "height": im.height, "mode": im.mode, "format": im.format, "metadata_keys": keys, "metadata_category": category})
    except Exception as exc:
        errors.append({"path": str(path), "error": type(exc).__name__})
chosen = {}
def choose(row): chosen[row["path"]] = row
for folder in sorted({r["folder"] for r in rows}):
    group = [r for r in rows if r["folder"] == folder]
    for row in group[:4]: choose(row)
    choose(min(group, key=lambda r: r["bytes"]))
    choose(max(group, key=lambda r: r["bytes"]))
    choose(max(group, key=lambda r: r["width"] * r["height"]))
for category in sorted({r["metadata_category"] for r in rows}):
    for row in [r for r in rows if r["metadata_category"] == category][:4]: choose(row)
fixture_root = pathlib.Path(tempfile.mkdtemp(prefix="omera-real-review-"))
fixture_rows = []
for idx, row in enumerate(chosen.values(), 1):
    source = pathlib.Path(row["path"])
    copied = fixture_root / ("sample-%03d%s" % (idx, source.suffix.lower()))
    original_digest = hashlib.sha256(source.read_bytes()).hexdigest()
    shutil.copy2(source, copied)
    copy_digest = hashlib.sha256(copied.read_bytes()).hexdigest()
    if original_digest != copy_digest: raise RuntimeError("Source changed during fixture copy")
    for ext in [".txt", ".json"]:
        sidecar = source.with_suffix(ext)
        if sidecar.is_file(): shutil.copy2(sidecar, copied.with_suffix(ext))
    fixture_rows.append({**row, "fixture": str(copied), "alias": copied.name, "source_sha256": original_digest})
summary = {"images": len(rows), "bytes": sum(r["bytes"] for r in rows), "folders": dict(collections.Counter(r["folder"] for r in rows)), "formats": dict(collections.Counter(r["format"] for r in rows)), "metadata_categories": dict(collections.Counter(r["metadata_category"] for r in rows)), "modes": dict(collections.Counter(r["mode"] for r in rows)), "maximum_pixels": max((r["width"] * r["height"] for r in rows), default=0), "header_errors": len(errors), "selected_fixtures": len(fixture_rows), "fixture_bytes": sum(r["bytes"] for r in fixture_rows), "fixture_root": str(fixture_root)}
(review / "real-sample-inventory-2026-10-05.json").write_text(json.dumps({"summary": summary, "images": rows, "errors": errors}, indent=2, ensure_ascii=False), encoding="utf-8")
(review / "real-test-fixtures-2026-10-05.json").write_text(json.dumps({"root": str(fixture_root), "fixtures": fixture_rows}, indent=2, ensure_ascii=False), encoding="utf-8")
print(json.dumps(summary, ensure_ascii=False))

