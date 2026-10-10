//! Bounded streaming primitives for media mirroring; no provider credentials or UI.

use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const TRANSFER_CHUNK_BYTES: usize = 64 * 1024;
pub const TRANSFER_BUFFER_COUNT: usize = 8;
const WAIT_STEP: Duration = Duration::from_millis(25);

struct Bucket {
    rate: u64,
    capacity: f64,
    tokens: f64,
    checked: Instant,
}

pub struct TransferControl {
    cancel: Arc<AtomicBool>,
    bucket: Mutex<Bucket>,
    available: Mutex<usize>,
    released: Condvar,
}

impl TransferControl {
    pub fn new(kb_per_sec: u64, cancel: Arc<AtomicBool>) -> Arc<Self> {
        let rate = kb_per_sec.saturating_mul(1024);
        let capacity = rate.saturating_mul(2) as f64;
        Arc::new(Self {
            cancel,
            bucket: Mutex::new(Bucket {
                rate,
                capacity,
                tokens: capacity,
                checked: Instant::now(),
            }),
            available: Mutex::new(TRANSFER_BUFFER_COUNT),
            released: Condvar::new(),
        })
    }

    pub fn check_cancelled(&self) -> io::Result<()> {
        if self.cancel.load(Ordering::SeqCst) {
            // Interrupted is retried by io::copy (including HTTP body writers).
            // A cancellation must terminate the stream instead of retrying it.
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Media sync cancelled",
            ))
        } else {
            Ok(())
        }
    }

    /// Reserve memory before allocating a buffer. Waiting remains cancellable.
    pub fn acquire(self: &Arc<Self>) -> io::Result<TransferLease> {
        let mut available = self.available.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            self.check_cancelled()?;
            if *available > 0 {
                *available -= 1;
                drop(available);
                return Ok(TransferLease {
                    control: Arc::clone(self),
                    buffer: vec![0; TRANSFER_CHUNK_BYTES],
                });
            }
            available = self
                .released
                .wait_timeout(available, WAIT_STEP)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    fn throttle(&self, bytes: usize) -> io::Result<()> {
        let mut remaining = bytes as f64;
        while remaining > 0.0 {
            self.check_cancelled()?;
            let wait = {
                let mut bucket = self.bucket.lock().unwrap_or_else(|e| e.into_inner());
                if bucket.rate == 0 {
                    return Ok(());
                }
                let now = Instant::now();
                let elapsed = now.duration_since(bucket.checked).as_secs_f64();
                bucket.checked = now;
                bucket.tokens = (bucket.tokens + elapsed * bucket.rate as f64).min(bucket.capacity);
                let consumed = remaining.min(bucket.tokens);
                remaining -= consumed;
                bucket.tokens -= consumed;
                Duration::from_secs_f64(
                    (remaining / bucket.rate as f64).min(WAIT_STEP.as_secs_f64()),
                )
            };
            if !wait.is_zero() {
                std::thread::sleep(wait);
            }
        }
        self.check_cancelled()
    }
}

pub struct TransferLease {
    control: Arc<TransferControl>,
    buffer: Vec<u8>,
}

impl TransferLease {
    pub fn hash(&mut self, mut source: impl Read) -> io::Result<String> {
        let mut digest = Sha256::new();
        loop {
            self.control.check_cancelled()?;
            let length = source.read(&mut self.buffer)?;
            self.control.check_cancelled()?;
            if length == 0 {
                break;
            }
            digest.update(&self.buffer[..length]);
        }
        Ok(hex::encode(digest.finalize()))
    }

    pub fn copy(&mut self, mut source: impl Read, mut destination: impl Write) -> io::Result<u64> {
        let mut total = 0;
        loop {
            self.control.check_cancelled()?;
            let length = source.read(&mut self.buffer)?;
            self.control.throttle(length)?;
            if length == 0 {
                break;
            }
            destination.write_all(&self.buffer[..length])?;
            total += length as u64;
        }
        self.control.check_cancelled()?;
        Ok(total)
    }

    /// Keep the lease alive while the provider consumes this bounded reader.
    pub fn reader<R: Read>(&self, source: R) -> TransferReader<'_, R> {
        TransferReader {
            source,
            control: &self.control,
        }
    }
}

impl Drop for TransferLease {
    fn drop(&mut self) {
        // Free the buffer before returning its budget to another worker.
        self.buffer = Vec::new();
        *self
            .control
            .available
            .lock()
            .unwrap_or_else(|e| e.into_inner()) += 1;
        self.control.released.notify_one();
    }
}

pub struct TransferReader<'a, R> {
    source: R,
    control: &'a TransferControl,
}

impl<R: Read> Read for TransferReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.control.check_cancelled()?;
        let length = output.len().min(TRANSFER_CHUNK_BYTES);
        let read = self.source.read(&mut output[..length])?;
        self.control.throttle(read)?;
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn large_stream_copy_and_digest_use_fixed_chunks() {
        struct Bounded(ReadCounter);
        struct ReadCounter {
            remaining: u64,
            max: usize,
        }
        impl Read for Bounded {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                self.0.max = self.0.max.max(output.len());
                assert!(output.len() <= TRANSFER_CHUNK_BYTES);
                let length = output.len().min(self.0.remaining as usize);
                output[..length].fill(0);
                self.0.remaining -= length as u64;
                Ok(length)
            }
        }
        let control = TransferControl::new(0, Arc::new(AtomicBool::new(false)));
        let mut lease = control.acquire().unwrap();
        let bytes = 128 * 1024 * 1024;
        let mut source = Bounded(ReadCounter {
            remaining: bytes,
            max: 0,
        });
        assert_eq!(lease.copy(&mut source, io::sink()).unwrap(), bytes);
        assert_eq!(source.0.max, TRANSFER_CHUNK_BYTES);
        assert_eq!(lease.buffer.len(), TRANSFER_CHUNK_BYTES);
        assert_eq!(
            lease.hash(&b"abc"[..]).unwrap(),
            hex::encode(Sha256::digest(b"abc"))
        );
    }

    #[test]
    fn shared_budget_blocks_extra_buffer_and_cancels_its_wait() {
        let cancel = Arc::new(AtomicBool::new(false));
        let control = TransferControl::new(0, Arc::clone(&cancel));
        let leases: Vec<_> = (0..TRANSFER_BUFFER_COUNT)
            .map(|_| control.acquire().unwrap())
            .collect();
        assert_eq!(*control.available.lock().unwrap(), 0);
        assert_eq!(
            leases.iter().map(|lease| lease.buffer.len()).sum::<usize>(),
            512 * 1024
        );
        let worker_control = Arc::clone(&control);
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            send.send(()).unwrap();
            worker_control.acquire().err().unwrap().kind()
        });
        receive.recv().unwrap();
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(worker.join().unwrap(), io::ErrorKind::ConnectionAborted);
        drop(leases);
        assert_eq!(*control.available.lock().unwrap(), TRANSFER_BUFFER_COUNT);
    }

    #[test]
    fn cancellation_interrupts_throttle_without_retaining_bucket_lock() {
        let cancel = Arc::new(AtomicBool::new(false));
        let control = TransferControl::new(1, Arc::clone(&cancel));
        // Exhaust the initial two-second allowance deterministically.
        control.bucket.lock().unwrap().tokens = 0.0;
        let worker_control = Arc::clone(&control);
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            send.send(()).unwrap();
            worker_control
                .throttle(TRANSFER_CHUNK_BYTES)
                .unwrap_err()
                .kind()
        });
        receive.recv().unwrap();
        cancel.store(true, Ordering::SeqCst);
        assert_eq!(worker.join().unwrap(), io::ErrorKind::ConnectionAborted);
        assert!(control.bucket.try_lock().is_ok());
        assert!(control.acquire().is_err());
    }

    #[test]
    fn cancellation_during_read_stops_hash_and_standard_stream_copy() {
        struct CancellingReader(Arc<AtomicBool>);
        impl Read for CancellingReader {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                output[0] = 1;
                self.0.store(true, Ordering::SeqCst);
                Ok(1)
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let control = TransferControl::new(0, Arc::clone(&cancel));
        let mut lease = control.acquire().unwrap();
        assert_eq!(
            lease
                .hash(CancellingReader(Arc::clone(&cancel)))
                .unwrap_err()
                .kind(),
            io::ErrorKind::ConnectionAborted
        );
        cancel.store(false, Ordering::SeqCst);
        let mut reader = lease.reader(CancellingReader(cancel));
        assert_eq!(
            io::copy(&mut reader, &mut io::sink()).unwrap_err().kind(),
            io::ErrorKind::ConnectionAborted
        );
    }
}
