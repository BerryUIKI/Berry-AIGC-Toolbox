//! Metadata detection and extraction for AI-generated image formats.
//!
//! [`detect_container`] sniffs a file's container from its magic bytes.
//! [`extract_metadata`] is the format-dispatch entry point used by the scan
//! engine: given a container and a file path it returns structured metadata or
//! `None`. Embedded metadata (PNGInfo, EXIF) is tried first; when a file carries
//! none, a sibling `.txt` sidecar is used as a fallback.

use std::path::Path;

use omera_domain::{Container, ExtractedMetadata, MetadataFormat};

pub mod comfyui;
pub mod container;
pub mod easydiffusion;
pub mod exif;
pub mod fooocus;
pub mod invokeai;
pub mod lora;
pub mod novelai;
pub mod parameters;
pub mod pnginfo;
pub mod sidecar;
pub mod video;

pub use container::detect_container;

/// Extract structured metadata from a media file, dispatching on its container.
///
/// Returns `None` when the file carries no recognizable metadata. Embedded
/// metadata wins; a `.txt` sidecar is only consulted as a fallback.
pub fn extract_metadata(container: Container, path: &Path) -> Option<ExtractedMetadata> {
    let embedded = match container {
        Container::Png => extract_png_metadata(path),
        Container::Jpeg | Container::WebP | Container::Avif => extract_exif_metadata(path),
        Container::Mp4 => video::extract_mp4_metadata(path),
        Container::Webm => video::extract_webm_metadata(path),
        Container::Txt => None,
    };
    embedded.or_else(|| extract_sidecar_metadata(path))
}

/// Extract metadata from a PNG file across all supported generator formats.
fn extract_png_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    let chunks = pnginfo::read_text_chunks_from_reader(reader).ok()?;

    // 1. Look for ComfyUI `prompt` / `workflow`
    for chunk in &chunks {
        if chunk.keyword == "prompt" || chunk.keyword == "workflow" {
            if let Some(meta) = comfyui::parse_comfyui(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 2. Look for InvokeAI chunks (`sd-metadata`, `invokeai_metadata`)
    for chunk in &chunks {
        if chunk.keyword == "sd-metadata"
            || chunk.keyword == "invokeai_metadata"
            || chunk.keyword == "invokeai"
        {
            if let Some(meta) = invokeai::parse_invokeai(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 3. Look for Stable Swarm
    for chunk in &chunks {
        if chunk.keyword == "sui_image_params" {
            if let Some(meta) = easydiffusion::parse_stableswarm(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 4. Look for NovelAI / EasyDiffusion in `Comment`
    let comment_chunk = chunks.iter().find(|c| c.keyword == "Comment");
    let description_chunk = chunks.iter().find(|c| c.keyword == "Description");
    if let Some(c) = comment_chunk {
        if let Some(meta) =
            novelai::parse_novelai(&c.text, description_chunk.map(|d| d.text.as_str()))
        {
            return Some(meta);
        }
        if let Some(meta) = easydiffusion::parse_easydiffusion(&c.text) {
            return Some(meta);
        }
        if let Some(meta) = comfyui::parse_comfyui(&c.text) {
            return Some(meta);
        }
    }

    // 5. Look for `parameters` chunk (A1111 / Fooocus / EasyDiffusion / ComfyUI)
    if let Some(param_chunk) = chunks.iter().find(|c| c.keyword == "parameters") {
        let text = &param_chunk.text;
        // Check Fooocus
        if let Some(meta) = fooocus::parse_fooocus(text) {
            return Some(meta);
        }
        // Check JSON formats (ComfyUI / EasyDiffusion)
        if text.trim_start().starts_with('{') {
            if let Some(meta) = comfyui::parse_comfyui(text) {
                return Some(meta);
            }
            if let Some(meta) = easydiffusion::parse_easydiffusion(text) {
                return Some(meta);
            }
            if let Some(meta) = novelai::parse_novelai(text, None) {
                return Some(meta);
            }
        }
        // Default to A1111
        return Some(from_parameters(text.clone()));
    }

    // 6. Description chunk alone (NovelAI / StableDiffusion fallback)
    if let Some(desc) = description_chunk {
        if !desc.text.trim().is_empty() {
            return Some(ExtractedMetadata {
                format: MetadataFormat::NovelAI,
                parameters: Some(desc.text.clone()),
                raw: Some(desc.text.clone()),
                prompt: Some(desc.text.trim().to_string()),
                negative_prompt: None,
                width: None,
                height: None,
                seed: None,
                steps: None,
                cfg_scale: None,
                sampler: None,
                model_name: None,
                model_hash: None,
                duration_seconds: None,
                fps: None,
                video_codec: None,
            });
        }
    }

    None
}

/// Extract dimensions + generator name from a JPEG/WebP file's EXIF block.
fn extract_exif_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let info = exif::read_exif(path).ok()??;
    let software = info.software.as_deref()?;
    let format = exif::infer_format(software)?;
    Some(ExtractedMetadata {
        format,
        parameters: None,
        raw: info.software,
        prompt: None,
        negative_prompt: None,
        width: info.width,
        height: info.height,
        seed: None,
        steps: None,
        cfg_scale: None,
        sampler: None,
        model_name: None,
        model_hash: None,
        duration_seconds: None,
        fps: None,
        video_codec: None,
    })
}

/// Read a sibling `<file>.txt` or `<file>.json` and parse it.
pub fn extract_sidecar_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let text = sidecar::read_sidecar(path).or_else(|| {
        let json_path = path.with_extension("json");
        if json_path.is_file() {
            std::fs::read_to_string(&json_path).ok()
        } else {
            None
        }
    })?;
    if let Some(meta) = comfyui::parse_comfyui(&text) {
        return Some(meta);
    }
    if let Some(meta) = fooocus::parse_fooocus(&text) {
        return Some(meta);
    }
    Some(from_parameters(text))
}

/// Build [`ExtractedMetadata`] from an A1111-style parameter string.
fn from_parameters(parameters: String) -> ExtractedMetadata {
    let parsed = parameters::parse_parameters(&parameters);
    ExtractedMetadata {
        format: MetadataFormat::A1111,
        parameters: Some(parameters),
        raw: None,
        prompt: parsed.prompt,
        negative_prompt: parsed.negative_prompt,
        width: parsed.width,
        height: parsed.height,
        seed: parsed.seed,
        steps: parsed.steps,
        cfg_scale: parsed.cfg_scale,
        sampler: parsed.sampler,
        model_name: parsed.model_name,
        model_hash: parsed.model_hash,
        duration_seconds: None,
        fps: None,
        video_codec: None,
    }
}

const EXACT_KEYWORDS: &[&str] = &[
    "nsfw",
    "nude",
    "naked",
    "nipples",
    "pussy",
    "penis",
    "vagina",
    "uncensored",
    "explicit",
    "hentai",
    "erotic",
    "sex",
    "blowjob",
    "fellatio",
    "penetration",
    "orgasm",
    "cum",
    "rating:explicit",
    "rating:questionable",
    "rating:e",
    "rating:q",
];

/// Check whether a text slice contains adult / NSFW keywords or explicit rating tags.
fn text_contains_nsfw(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains("rating:explicit")
        || lower.contains("rating:questionable")
        || lower.contains("nsfw")
    {
        return true;
    }

    // Normalize "rating: " with space to "rating:" so "rating: explicit", "rating: e", "rating: q" match.
    let normalized = lower.replace("rating: ", "rating:");

    for token in normalized.split(|c: char| !c.is_alphanumeric() && c != ':') {
        let t = token.trim();
        if !t.is_empty() && EXACT_KEYWORDS.contains(&t) {
            return true;
        }
    }

    false
}

/// Check if a rating value represents an adult / explicit rating.
fn is_explicit_rating(val: &str) -> bool {
    let lower = val.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "explicit"
            | "questionable"
            | "e"
            | "q"
            | "nsfw"
            | "rating:explicit"
            | "rating:questionable"
            | "rating:e"
            | "rating:q"
    )
}

/// Check if metadata carries an explicit rating in structured fields outside the negative prompt.
fn has_explicit_rating(meta: &ExtractedMetadata) -> bool {
    // 1. Top-level rating field in JSON metadata (NovelAI / Swarm / JSON containers)
    for text in [meta.raw.as_deref(), meta.parameters.as_deref()]
        .into_iter()
        .flatten()
    {
        let trimmed = text.trim();
        if trimmed.starts_with('{') {
            if let Ok(serde_json::Value::Object(map)) =
                serde_json::from_str::<serde_json::Value>(trimmed)
            {
                if let Some(r) = map.get("rating").and_then(|v| v.as_str()) {
                    if is_explicit_rating(r) {
                        return true;
                    }
                }
            }
        }
    }

    // 2. Settings line rating in A1111 parameter strings (e.g. "Rating: explicit")
    if let Some(ref params) = meta.parameters {
        for line in params.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Negative prompt:")
                || trimmed.starts_with("Negative Prompt:")
                || trimmed.starts_with("negative prompt:")
                || trimmed.starts_with("Negative:")
                || trimmed.starts_with("negative_prompt:")
            {
                continue;
            }
            for segment in trimmed.split(',') {
                if let Some((k, v)) = segment.split_once(':') {
                    if k.trim().eq_ignore_ascii_case("rating") && is_explicit_rating(v) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Extract candidate positive text from raw/unstructured metadata during fallback,
/// explicitly stripping negative-prompt sections.
fn extract_fallback_positive_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // A. Check JSON formats
    if trimmed.starts_with('{') {
        if let Ok(serde_json::Value::Object(map)) =
            serde_json::from_str::<serde_json::Value>(trimmed)
        {
            if let Some(p) = map
                .get("prompt")
                .or_else(|| map.get("positive_prompt"))
                .or_else(|| map.get("positive"))
                .and_then(|v| v.as_str())
            {
                return Some(p.to_string());
            }
        }
        if let Some(comfy) = comfyui::parse_comfyui(trimmed) {
            return comfy.prompt;
        }
        return None;
    }

    // B. Check Fooocus format
    if let Some(fooocus_meta) = fooocus::parse_fooocus(trimmed) {
        if fooocus_meta.prompt.is_some() {
            return fooocus_meta.prompt;
        }
    }

    // C. Check A1111 / parameters parser
    let parsed = parameters::parse_parameters(trimmed);
    if parsed.prompt.is_some() {
        return parsed.prompt;
    }

    // D. If neither positive nor negative was separated, check for any negative marker
    const NEG_MARKERS: &[&str] = &[
        "Negative prompt:",
        "Negative Prompt:",
        "negative prompt:",
        "Negative:",
        "negative_prompt:",
    ];
    for marker in NEG_MARKERS {
        if let Some(pos) = trimmed.find(marker) {
            let positive_slice = trimmed[..pos].trim();
            if !positive_slice.is_empty() {
                return Some(positive_slice.to_string());
            } else {
                return None;
            }
        }
    }

    Some(trimmed.to_string())
}

/// Inspect structured metadata prompt or explicit rating fields to auto-detect adult / NSFW content.
///
/// Negative prompt exclusions are excluded from positive classification.
/// Explicit rating fields follow the established policy (e.g. `rating:explicit`, `rating:e`,
/// `rating:questionable`, `rating:q` flag as NSFW; `rating:safe`, `rating:general`, `rating:s` do not).
/// When structured prompts are absent or malformed, falls back to deliberate inspection
/// of candidate positive text while preserving negative exclusions.
pub fn detect_nsfw_from_metadata(meta: &ExtractedMetadata) -> bool {
    // 1. Structured positive prompt check
    if let Some(ref p) = meta.prompt {
        if text_contains_nsfw(p) {
            return true;
        }
    }

    // 2. Structured explicit rating fields check (JSON rating or settings Rating: ...)
    if has_explicit_rating(meta) {
        return true;
    }

    // 3. Fallback when structured positive prompt is absent
    // If a structured negative prompt was already identified, the positive prompt was empty/absent;
    // negative prompt exclusions must not count as positive NSFW signals.
    if meta.prompt.is_none() && meta.negative_prompt.is_none() {
        for text in [meta.parameters.as_deref(), meta.raw.as_deref()]
            .into_iter()
            .flatten()
        {
            if let Some(pos_text) = extract_fallback_positive_text(text) {
                if text_contains_nsfw(&pos_text) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tex(keyword: &str, text: &str) -> Vec<u8> {
        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.extend_from_slice(text.as_bytes());
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(b"tEXt");
        out.extend_from_slice(&data);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn png_with_parameters(text: &str) -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("parameters", text));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn plain_png() -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    /// Minimal JPEG with an EXIF APP1 segment (`ComfyUI` software + dimensions).
    fn exif_jpeg() -> Vec<u8> {
        let software = "ComfyUI";
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&0x002A_u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&1u16.to_le_bytes()); // single entry: Software
        tiff.extend_from_slice(&0x0131_u16.to_le_bytes());
        tiff.extend_from_slice(&2u16.to_le_bytes());
        tiff.extend_from_slice(&(software.len() as u32 + 1).to_le_bytes());
        tiff.extend_from_slice(&26u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD offset
        tiff.extend_from_slice(software.as_bytes());
        tiff.push(0);

        let marker = b"Exif\0\0";
        let app1_len = 2 + marker.len() + tiff.len();
        let mut out = vec![0xFF, 0xD8];
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&(app1_len as u16).to_be_bytes());
        out.extend_from_slice(marker);
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    /// Helper: write `bytes` to a fresh temp subdirectory and return the path.
    fn temp_file(dir: &str, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("omera-meta-{}", std::process::id()))
            .join(dir);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn extracts_png_metadata() {
        let path = temp_file(
            "png",
            "test.png",
            &png_with_parameters(
                "a cat\nNegative prompt: blurry\nSteps: 20, Seed: 42, Size: 512x768",
            ),
        );

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a cat"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
        assert_eq!(meta.width, Some(512));
        assert_eq!(meta.height, Some(768));

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_exif_metadata_from_jpeg() {
        let path = temp_file("exif", "comfy.jpg", &exif_jpeg());
        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(meta.raw.as_deref(), Some("ComfyUI"));
        assert_eq!(meta.prompt, None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_jpeg_falls_back_to_sidecar() {
        let path = temp_file("jpeg-sidecar", "plain.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        std::fs::write(
            path.with_extension("txt"),
            "a landscape\nSteps: 12, Sampler: DPM++ 2M",
        )
        .unwrap();

        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a landscape"));
        assert_eq!(meta.steps, Some(12));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_png_falls_back_to_sidecar() {
        let path = temp_file("png-sidecar", "plain.png", &plain_png());
        std::fs::write(path.with_extension("txt"), "just a prompt").unwrap();

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("just a prompt"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn images_with_no_metadata_return_none() {
        let path = temp_file("none", "photo.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        assert_eq!(extract_metadata(Container::Jpeg, &path), None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_comfyui_png_metadata() {
        let json = r#"{"3":{"class_type":"KSampler","inputs":{"seed":123,"steps":20,"cfg":8.0,"sampler_name":"euler","positive":["4",0]}},"4":{"class_type":"CLIPTextEncode","inputs":{"text":"comfy forest"}}}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("prompt", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-comfy", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(meta.prompt.as_deref(), Some("comfy forest"));
        assert_eq!(meta.steps, Some(20));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_novelai_png_metadata() {
        let json = r#"{"prompt":"anime warrior","uc":"lowres","steps":28,"scale":6.0,"seed":42,"sampler":"k_euler"}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("Comment", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-novelai", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::NovelAI);
        assert_eq!(meta.prompt.as_deref(), Some("anime warrior"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("lowres"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_invokeai_png_metadata() {
        let json = r#"{"positive_prompt":"cyberpunk room","steps":35,"cfg_scale":7.0,"seed":999}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("sd-metadata", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-invokeai", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::InvokeAI);
        assert_eq!(meta.prompt.as_deref(), Some("cyberpunk room"));
        assert_eq!(meta.steps, Some(35));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_fooocus_png_metadata() {
        let text = "Prompt: dragon flying over mountains\nResolution: (1024, 1024)\nSteps: 30\nBase Model: sd_xl_base.safetensors";
        let path = temp_file("png-fooocus", "test.png", &png_with_parameters(text));
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::Fooocus);
        assert_eq!(meta.prompt.as_deref(), Some("dragon flying over mountains"));
        assert_eq!(meta.steps, Some(30));
        assert_eq!(meta.model_name.as_deref(), Some("sd_xl_base.safetensors"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn videos_can_extract_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let mp4_path = dir.path().join("x.mp4");
        let txt_path = dir.path().join("x.txt");

        std::fs::write(&mp4_path, b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00").unwrap();
        std::fs::write(
            &txt_path,
            "AnimateDiff animation prompt\nSteps: 20, Sampler: Euler, Seed: 42",
        )
        .unwrap();

        let meta = extract_metadata(Container::Mp4, &mp4_path).expect("video metadata extracted");
        assert_eq!(meta.prompt.as_deref(), Some("AnimateDiff animation prompt"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
    }

    #[test]
    fn test_detect_nsfw_from_metadata() {
        let mut meta = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: None,
            raw: None,
            prompt: Some("1girl, masterpiece, nsfw, highly detailed".to_string()),
            negative_prompt: None,
            width: None,
            height: None,
            seed: None,
            steps: None,
            cfg_scale: None,
            sampler: None,
            model_name: None,
            model_hash: None,
            duration_seconds: None,
            fps: None,
            video_codec: None,
        };
        assert!(detect_nsfw_from_metadata(&meta));

        meta.prompt = Some("cyberpunk city, neon lights, unisex clothing".to_string());
        assert!(!detect_nsfw_from_metadata(&meta));

        meta.prompt = Some("beautiful portrait, nude, soft lighting".to_string());
        assert!(detect_nsfw_from_metadata(&meta));

        meta.prompt = Some("anime girl, rating:explicit".to_string());
        assert!(detect_nsfw_from_metadata(&meta));
    }

    #[test]
    fn negative_prompt_exclusions_must_not_classify_a_safe_prompt_as_nsfw() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("safe-landscape.png");
        let png_bytes = png_with_parameters(
            "landscape, mountain, daylight\nNegative prompt: nsfw, nude\nSteps: 20, Sampler: Euler, Seed: 123, Size: 16x16",
        );
        std::fs::write(&path, png_bytes).unwrap();

        let metadata = extract_metadata(Container::Png, &path).expect("metadata extracted");
        assert_eq!(
            metadata.prompt.as_deref(),
            Some("landscape, mountain, daylight")
        );
        assert_eq!(metadata.negative_prompt.as_deref(), Some("nsfw, nude"));
        assert!(
            !detect_nsfw_from_metadata(&metadata),
            "Safe positive prompt with adult terms in negative prompt must remain safe"
        );
    }

    #[test]
    fn adult_positive_prompt_remains_flagged_with_safe_negative() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("adult-portrait.png");
        let png_bytes = png_with_parameters(
            "1girl, nude, masterpiece\nNegative prompt: lowres, blurry, safe\nSteps: 20, Sampler: Euler, Size: 512x512",
        );
        std::fs::write(&path, png_bytes).unwrap();

        let metadata = extract_metadata(Container::Png, &path).expect("metadata extracted");
        assert_eq!(metadata.prompt.as_deref(), Some("1girl, nude, masterpiece"));
        assert!(
            detect_nsfw_from_metadata(&metadata),
            "Adult positive prompt must remain flagged even with safe negative prompt"
        );
    }

    #[test]
    fn explicit_rating_fields_follow_policy() {
        let mut meta = ExtractedMetadata {
            format: MetadataFormat::A1111,
            ..Default::default()
        };

        // Adult rating tags in positive prompt flag as NSFW
        for adult_tag in &[
            "rating:explicit",
            "rating:questionable",
            "rating:e",
            "rating:q",
            "rating: explicit",
            "rating: questionable",
            "rating: e",
            "rating: q",
        ] {
            meta.prompt = Some(format!("anime girl, {adult_tag}"));
            assert!(
                detect_nsfw_from_metadata(&meta),
                "Tag {adult_tag} in positive prompt must flag as NSFW"
            );
        }

        // Safe rating tags in positive prompt do not flag as NSFW
        for safe_tag in &[
            "rating:safe",
            "rating:general",
            "rating:s",
            "rating:g",
            "rating: safe",
            "rating: general",
        ] {
            meta.prompt = Some(format!("anime girl, {safe_tag}"));
            meta.negative_prompt =
                Some("rating:explicit, rating:questionable, rating:e, nsfw, nude".to_string());
            assert!(
                !detect_nsfw_from_metadata(&meta),
                "Safe tag {safe_tag} with adult negative prompt must remain safe"
            );
        }

        // JSON metadata with top-level rating
        let json_explicit = ExtractedMetadata {
            format: MetadataFormat::NovelAI,
            raw: Some(
                r#"{"prompt": "anime girl", "rating": "explicit", "uc": "nsfw"}"#.to_string(),
            ),
            prompt: Some("anime girl".to_string()),
            negative_prompt: Some("nsfw".to_string()),
            ..Default::default()
        };
        assert!(
            detect_nsfw_from_metadata(&json_explicit),
            "Top-level explicit rating in JSON must flag as NSFW"
        );

        let json_safe = ExtractedMetadata {
            format: MetadataFormat::NovelAI,
            raw: Some(r#"{"prompt": "anime girl", "rating": "safe", "uc": "nsfw, nude, rating:explicit"}"#.to_string()),
            prompt: Some("anime girl".to_string()),
            negative_prompt: Some("nsfw, nude, rating:explicit".to_string()),
            ..Default::default()
        };
        assert!(
            !detect_nsfw_from_metadata(&json_safe),
            "Top-level safe rating in JSON must remain safe despite adult uc"
        );

        // A1111 parameters with Rating: explicit in settings line
        let params_meta = ExtractedMetadata {
            format: MetadataFormat::A1111,
            prompt: Some("anime girl".to_string()),
            negative_prompt: Some("lowres".to_string()),
            parameters: Some(
                "anime girl\nNegative prompt: lowres\nSteps: 20, Rating: explicit".to_string(),
            ),
            ..Default::default()
        };
        assert!(
            detect_nsfw_from_metadata(&params_meta),
            "Rating: explicit in settings line must flag as NSFW"
        );
    }

    #[test]
    fn malformed_and_raw_fallback_behavior() {
        // Raw text with adult keywords and no negative prompt
        let raw_adult = ExtractedMetadata {
            format: MetadataFormat::A1111,
            raw: Some("an uncensored nude portrait, masterpiece".to_string()),
            ..Default::default()
        };
        assert!(
            detect_nsfw_from_metadata(&raw_adult),
            "Fallback raw text with adult keywords must flag"
        );

        // Raw text with safe keywords
        let raw_safe = ExtractedMetadata {
            format: MetadataFormat::A1111,
            raw: Some("a peaceful landscape with green trees".to_string()),
            ..Default::default()
        };
        assert!(
            !detect_nsfw_from_metadata(&raw_safe),
            "Fallback raw text with safe content must not flag"
        );

        // Raw text with negative prompt marker isolating adult terms
        let raw_neg = ExtractedMetadata {
            format: MetadataFormat::A1111,
            raw: Some("a peaceful landscape\nNegative prompt: nsfw, nude".to_string()),
            ..Default::default()
        };
        assert!(
            !detect_nsfw_from_metadata(&raw_neg),
            "Fallback raw text with adult terms only in negative prompt must not flag"
        );

        // Parameters with adult positive and safe negative in unparsed fallback
        let params_adult = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("an erotic painting\nNegative prompt: lowres".to_string()),
            ..Default::default()
        };
        assert!(
            detect_nsfw_from_metadata(&params_adult),
            "Fallback parameters with adult positive must flag"
        );

        // Negative-only metadata with no positive prompt
        let neg_only = ExtractedMetadata {
            format: MetadataFormat::A1111,
            negative_prompt: Some("nsfw, nude, bad anatomy".to_string()),
            ..Default::default()
        };
        assert!(
            !detect_nsfw_from_metadata(&neg_only),
            "Metadata with only negative prompt must not flag as NSFW"
        );

        // ComfyUI JSON raw text in fallback: safe positive, adult negative
        let comfy_json_safe = r#"{
            "prompt": {
                "3": {
                    "class_type": "KSampler",
                    "inputs": {
                        "positive": ["6", 0],
                        "negative": ["7", 0]
                    }
                },
                "6": {
                    "class_type": "CLIPTextEncode",
                    "inputs": {"text": "a cute kitten"}
                },
                "7": {
                    "class_type": "CLIPTextEncode",
                    "inputs": {"text": "nsfw, nude"}
                }
            }
        }"#;
        let comfy_meta_safe = ExtractedMetadata {
            format: MetadataFormat::ComfyUI,
            raw: Some(comfy_json_safe.to_string()),
            ..Default::default()
        };
        assert!(
            !detect_nsfw_from_metadata(&comfy_meta_safe),
            "ComfyUI raw JSON fallback with safe positive must not flag"
        );

        // ComfyUI JSON raw text in fallback: adult positive
        let comfy_json_adult = r#"{
            "prompt": {
                "3": {
                    "class_type": "KSampler",
                    "inputs": {
                        "positive": ["6", 0],
                        "negative": ["7", 0]
                    }
                },
                "6": {
                    "class_type": "CLIPTextEncode",
                    "inputs": {"text": "nude model"}
                },
                "7": {
                    "class_type": "CLIPTextEncode",
                    "inputs": {"text": "lowres"}
                }
            }
        }"#;
        let comfy_meta_adult = ExtractedMetadata {
            format: MetadataFormat::ComfyUI,
            raw: Some(comfy_json_adult.to_string()),
            ..Default::default()
        };
        assert!(
            detect_nsfw_from_metadata(&comfy_meta_adult),
            "ComfyUI raw JSON fallback with adult positive must flag"
        );
    }
}
