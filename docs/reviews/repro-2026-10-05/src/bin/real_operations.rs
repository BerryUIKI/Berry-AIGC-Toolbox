#[path = "../../../../../src-tauri/src/cloud_backup.rs"]
mod cloud_backup;

use omera_domain::*;
use omera_storage::Database;
use serde_json::json;
use std::{fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&args[1])?)?;
    let source = Path::new(manifest["fixtures"][0]["fixture"].as_str().unwrap());
    let temp = tempfile::tempdir()?;
    let mut codecs = Vec::new();
    for format in [TransformFormat::Png, TransformFormat::Jpeg, TransformFormat::Webp] {
        let output = omera_scan::transform_file_staged(source, &temp.path().join("stage"), &TransformSpec {
            format, max_edge:Some(128), quality:Some(75), metadata_policy:TransformMetadataPolicy::StripAll,
            ..Default::default()
        })?;
        let img = image::open(&output)?;
        codecs.push(json!({"format":format,"decoded":true,"dimensions":[img.width(),img.height()],"bytes":fs::metadata(&output)?.len()}));
    }
    let db_path = temp.path().join("operations.db");
    let db = Database::connect(&db_path)?;
    let managed = temp.path().join("managed"); fs::create_dir(&managed)?;
    let folder = db.add_folder_with_mode(managed.to_str().unwrap(), "managed",None,None,None,false)?;
    let request = ImportTransformRequest {managed_destination_id:folder.id,source_paths:vec![source.to_str().unwrap().into()],
        spec:TransformSpec{format:TransformFormat::Png,max_edge:Some(128),..Default::default()},source_disposition:ImportSourceDisposition::Keep};
    let first = omera_scan::execute_managed_import_transform(&db,&request,None::<fn(usize,usize,&str)>)?;
    let second = omera_scan::execute_managed_import_transform(&db,&request,None::<fn(usize,usize,&str)>)?;
    let files = db.list_files(folder.id)?;
    let distinct_paths = files.len()==2 && files[0].path!=files[1].path;
    let sources_preserved = source.exists();
    let mut skip_request = request.clone(); skip_request.spec.collision_policy=TransformCollisionPolicy::Skip;
    let skipped = omera_scan::execute_managed_import_transform(&db,&skip_request,None::<fn(usize,usize,&str)>)?;
    let ids:Vec<i64> = files.iter().filter_map(|f|f.id).collect();
    let export_dir=temp.path().join("exports"); fs::create_dir(&export_dir)?;
    let zip_path=export_dir.join("review.zip");
    let options = ExportOptions{file_ids:ids, format:ExportFormat::Original,quality:85,privacy:MetadataPrivacyMode::KeepAll,
        sidecar:ExportSidecar::JsonMetadata,filename_template:"{name}".into(),destination_path:zip_path.to_string_lossy().into_owned(),
        as_zip:true,max_edge:None,export_html_showcase:false,html_title:None};
    let export_summary=omera_scan::export::execute_batch_export(&db,&options,|_| {});
    let mut zip=zip::ZipArchive::new(fs::File::open(&zip_path)?)?;
    let mut zip_images_decoded=0;
    for index in 0..zip.len(){
        let mut member=zip.by_index(index)?;
        if member.name().ends_with(".png"){
            let mut bytes=Vec::new(); std::io::Read::read_to_end(&mut member,&mut bytes)?;
            if image::load_from_memory(&bytes).is_ok(){zip_images_decoded+=1;}
        }
    }
    let config=CloudBackupConfig{local_path:Some(temp.path().join("cloud").to_string_lossy().into_owned()),..Default::default()};
    let snapshot=cloud_backup::create_cloud_snapshot(&db,&config,temp.path(),None)?;
    let filename=snapshot.snapshot.unwrap().filename;
    let original_rows=db.list_files(folder.id)?.len();
    drop(db);
    let restored_path=temp.path().join("closed-restore.db");
    let restored=cloud_backup::restore_cloud_snapshot(&restored_path,&config,&filename)?;
    let reopened=Database::connect(&restored_path)?;
    let restored_rows=reopened.list_files(folder.id)?.len();
    let integrity:String=reopened.connection().query_row("PRAGMA integrity_check",[],|r|r.get(0))?;
    let results=json!({"codecs":codecs,"managed_import":{"first_succeeded":first.succeeded,"rename_succeeded":second.succeeded,
        "distinct_paths":distinct_paths,"skip_count":skipped.skipped,"sources_preserved":sources_preserved},
        "zip_export":{"success":export_summary.success,"exported":export_summary.total_exported,"failed":export_summary.total_failed,"decoded_images":zip_images_decoded},
        "local_cloud_snapshot":{"restore_success":restored.success,"original_rows":original_rows,"restored_rows":restored_rows,"integrity_check":integrity}});
    fs::write(&args[2],serde_json::to_vec_pretty(&results)?)?;
    println!("{}",results);
    Ok(())
}
