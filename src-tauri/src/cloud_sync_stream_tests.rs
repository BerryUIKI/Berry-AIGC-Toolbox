use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

#[test]
fn cancelled_throttled_local_transfer_preserves_source_and_previous_destination() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source.png");
    File::create(&source).unwrap().set_len(128 * 1024).unwrap();
    let destination = root.path().join("mirror");
    fs::create_dir(&destination).unwrap();
    let target = destination.join("media.png");
    fs::write(&target, b"previous complete version").unwrap();
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::LocalPath,
        local_path: Some(destination.to_string_lossy().into()),
        ..Default::default()
    };
    let item = SyncItem {
        local_path: source.clone(),
        remote_key: "media.png".into(),
        size_bytes: 128 * 1024,
        mtime_secs: 0,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let control = TransferControl::new(1, Arc::clone(&cancel));
    let (send, receive) = mpsc::channel();
    let worker = thread::spawn(move || {
        send.send(sync_single_item(&config, &CloudSyncOptions::default(), &item, &control).err())
            .unwrap();
    });
    // A 128 KiB transfer at 1 KiB/s cannot finish during this allowance.
    thread::sleep(Duration::from_millis(50));
    cancel.store(true, Ordering::SeqCst);
    assert!(receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap()
        .contains("cancelled"));
    worker.join().unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"previous complete version");
    assert_eq!(fs::metadata(&source).unwrap().len(), 128 * 1024);
    assert_eq!(fs::read_dir(&destination).unwrap().count(), 1);
}

#[test]
fn sparse_large_local_media_is_streamed_and_published_with_identical_digest() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("large.png");
    File::create(&source)
        .unwrap()
        .set_len(8 * 1024 * 1024)
        .unwrap();
    let destination = root.path().join("mirror");
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::LocalPath,
        local_path: Some(destination.to_string_lossy().into()),
        ..Default::default()
    };
    let item = SyncItem {
        local_path: source.clone(),
        remote_key: "nested/large.png".into(),
        size_bytes: 8 * 1024 * 1024,
        mtime_secs: 0,
    };
    let control = TransferControl::new(0, Arc::new(AtomicBool::new(false)));
    assert!(matches!(
        sync_single_item(&config, &CloudSyncOptions::default(), &item, &control).unwrap(),
        SyncOutcome::Uploaded(8_388_608)
    ));
    let mut lease = control.acquire().unwrap();
    assert_eq!(
        lease.hash(File::open(&source).unwrap()).unwrap(),
        lease
            .hash(File::open(destination.join(&item.remote_key)).unwrap())
            .unwrap()
    );
    assert!(source.exists());
}

#[test]
fn stalled_webdav_checksum_body_has_bounded_cancellation_completion() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source.png");
    fs::write(&source, b"different local!").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let (started_send, started_receive) = mpsc::channel();
    let (release_send, release_receive) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut head, _) = listener.accept().unwrap();
        let mut request = [0; 1024];
        head.read(&mut request).unwrap();
        head.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\n")
            .unwrap();
        drop(head);
        let (mut body, _) = listener.accept().unwrap();
        body.read(&mut request).unwrap();
        body.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\nx")
            .unwrap();
        started_send.send(()).unwrap();
        let _ = release_receive.recv_timeout(Duration::from_secs(5));
    });
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::WebDav,
        webdav_endpoint: Some(endpoint),
        ..Default::default()
    };
    let options = CloudSyncOptions {
        strategy: CloudSyncStrategy::Sha256Checksum,
        ..Default::default()
    };
    let item = SyncItem {
        local_path: source,
        remote_key: "media.png".into(),
        size_bytes: 16,
        mtime_secs: 0,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let control = TransferControl::new(0, Arc::clone(&cancel));
    let (send, receive) = mpsc::channel();
    let worker = thread::spawn(move || {
        send.send(sync_single_item(&config, &options, &item, &control).is_err())
            .unwrap();
    });
    started_receive
        .recv_timeout(Duration::from_secs(3))
        .unwrap();
    // Allow the consumer to enter its next socket read before cancelling.
    thread::sleep(Duration::from_millis(50));
    cancel.store(true, Ordering::SeqCst);
    assert!(receive.recv_timeout(Duration::from_secs(3)).unwrap());
    let _ = release_send.send(());
    worker.join().unwrap();
    server.join().unwrap();
}
