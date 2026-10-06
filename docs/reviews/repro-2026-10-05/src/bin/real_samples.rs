use omera_domain::*;
use omera_storage::Database;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, time::Instant};

fn prompt_present(path: &Path, container: Container) -> bool {
    omera_metadata::extract_metadata(container, path)
        .and_then(|m| m.prompt).is_some_and(|p| !p.trim().is_empty())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&args[1])?)?;
    let fixtures = manifest["fixtures"].as_array().unwrap();
    let work = tempfile::tempdir()?;
    let db_path = work.path().join("review.db");
    let db = Database::connect(&db_path)?;
    let folder = db.add_folder(&args[2])?;
    let scanner = omera_scan::scanner::Scanner::with_default_extractor(db_path.clone());
    let mut progress_events = 0;
    let scan = scanner.scan_folder(folder.id, Path::new(&args[2]), |_| progress_events += 1)?;
    let repeat_scan = scanner.scan_folder(folder.id, Path::new(&args[2]), |_| {})?;
    let indexed = db.list_files(folder.id)?;
    let with_prompt = indexed.iter().filter(|f| f.metadata.as_ref().and_then(|m| m.prompt.as_ref()).is_some_and(|p| !p.trim().is_empty())).count();
    let mut results = Vec::new();
    for (i, fixture) in fixtures.iter().enumerate() {
        let source = Path::new(fixture["fixture"].as_str().unwrap());
        let started = Instant::now();
        let decoded = image::open(source)?;
        let thumb = work.path().join(format!("thumb-{i}.webp"));
        let thumbnail_ok = omera_scan::thumbnail::generate_thumbnail(source, &thumb, 256).is_ok();
        let original = prompt_present(source, Container::Png);
        let spec = TransformSpec { format: TransformFormat::Png, max_edge: Some(512), ..Default::default() };
        let derivative = omera_scan::transform_file_staged(source, &work.path().join("stage"), &spec)?;
        let keep_supported_prompt = prompt_present(&derivative, Container::Png);
        let dims = image::image_dimensions(&derivative)?;
        let file = ImageFile {
            id: None, folder_id: folder.id, path: source.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(source)?.len(), modified_at: 1, container: Container::Png,
            metadata: omera_metadata::extract_metadata(Container::Png, source),
            rating: None, aesthetic_score: None, is_favorite: false, is_nsfw: false, stack_id: None, stack_order: 0,
        };
        let mut opts = ExportOptions { file_ids: vec![], format: ExportFormat::Original, quality: 85,
            privacy: MetadataPrivacyMode::KeepAll, sidecar: ExportSidecar::None,
            filename_template: "{name}".into(), destination_path: "unused".into(), as_zip: false,
            max_edge: None, export_html_showcase: false, html_title: None };
        let passthrough = omera_scan::export::process_single_image(&file, &opts, i)?;
        let byte_exact = passthrough.image_bytes == fs::read(source)?;
        opts.format = ExportFormat::Png; opts.max_edge = Some(512);
        let encoded = omera_scan::export::process_single_image(&file, &opts, i)?;
        let encoded_path = work.path().join(format!("export-{i}.png"));
        fs::write(&encoded_path, &encoded.image_bytes)?;
        let keep_all_prompt = prompt_present(&encoded_path, Container::Png);
        opts.privacy = MetadataPrivacyMode::StripPromptOnly; opts.sidecar = ExportSidecar::TextPrompt;
        let private = omera_scan::export::process_single_image(&file, &opts, i)?;
        let prompt_sidecar_leak = original && private.sidecar.as_ref().is_some_and(|(_, b)| !b.is_empty());
        let unchanged_source = hex::encode(Sha256::digest(fs::read(fixture["path"].as_str().unwrap())?)) == fixture["source_sha256"].as_str().unwrap();
        results.push(json!({"alias":fixture["alias"],"dimensions":[decoded.width(),decoded.height()],"thumbnail_ok":thumbnail_ok,
            "source_has_prompt":original,"keep_supported_has_prompt":keep_supported_prompt,"derivative_dimensions":[dims.0,dims.1],
            "keep_all_reencode_has_prompt":keep_all_prompt,"original_export_byte_exact":byte_exact,"strip_prompt_sidecar_leak":prompt_sidecar_leak,
            "original_sha256_unchanged":unchanged_source,"duration_ms":started.elapsed().as_millis()}));
    }
    let first = Path::new(fixtures[0]["fixture"].as_str().unwrap());
    let avif = omera_scan::transform_file_staged(first, &work.path().join("avif-only"), &TransformSpec {format:TransformFormat::Avif,max_edge:Some(128), ..Default::default()})?;
    let avif_thumb_ok = omera_scan::thumbnail::generate_thumbnail(&avif, &work.path().join("avif-thumb.webp"), 64).is_ok();
    let avif_folder = db.add_folder(avif.parent().unwrap().to_str().unwrap())?;
    let avif_scan = scanner.scan_folder(avif_folder.id, avif.parent().unwrap(), |_| {})?;
    let output = json!({"scan":scan,"repeat_scan":repeat_scan,"scan_progress_events":progress_events,
        "indexed_count":indexed.len(),"indexed_with_prompt":with_prompt,"fixtures":results,
        "avif":{"created":true,"thumbnail_ok":avif_thumb_ok,"scan_found":avif_scan.found,"scan_added":avif_scan.added}});
    fs::write(&args[3], serde_json::to_vec_pretty(&output)?)?;
    println!("{}", json!({"indexed":indexed.len(),"with_prompt":with_prompt,"fixture_count":results.len(),"avif":output["avif"]}));
    Ok(())
}
