use std::{
    collections::HashSet,
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::Duration,
};

#[cfg(not(target_has_atomic = "64"))]
use portable_atomic::AtomicU64;
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::AtomicU64;

use crate::display::human_readable_number;

/* -------------------------------------------------------------------------- */

pub const ORDERING: Ordering = Ordering::Relaxed;

const SPINNER_SLEEP_TIME: u64 = 100;
const PROGRESS_CHARS: [char; 4] = ['-', '\\', '|', '/'];
const PROGRESS_CHARS_LEN: usize = PROGRESS_CHARS.len();

pub trait ThreadSyncTrait<T> {
    fn set(&self, val: T);
    fn get(&self) -> T;
}

#[derive(Default)]
pub struct ThreadStringWrapper {
    inner: RwLock<String>,
}

impl ThreadSyncTrait<String> for ThreadStringWrapper {
    fn set(&self, val: String) {
        *self.inner.write().unwrap() = val;
    }

    fn get(&self) -> String {
        (*self.inner.read().unwrap()).clone()
    }
}

/* -------------------------------------------------------------------------- */

// creating an enum this way allows to have simpler syntax compared to a Mutex or a RwLock
#[allow(non_snake_case)]
pub mod Operation {
    pub const INDEXING: u8 = 0;
    pub const PREPARING: u8 = 1;
}

#[derive(Default)]
pub struct PAtomicInfo {
    pub num_files: AtomicUsize,
    pub total_file_size: AtomicU64,
    pub state: AtomicU8,
    pub current_path: ThreadStringWrapper,
}

impl PAtomicInfo {
    pub fn clear_state(&self, dir: &Path) {
        self.state.store(Operation::INDEXING, ORDERING);
        let dir_name = dir.to_string_lossy().to_string();
        self.current_path.set(dir_name);
        self.total_file_size.store(0, ORDERING);
        self.num_files.store(0, ORDERING);
    }
}

#[derive(Default)]
pub struct RuntimeErrors {
    pub no_permissions: HashSet<String>,
    pub file_not_found: HashSet<String>,
    pub unknown_error: HashSet<String>,
    pub abort: bool,
}

/* -------------------------------------------------------------------------- */

fn format_preparing_str(prog_char: char, data: &PAtomicInfo, output_display: &str) -> String {
    let path_in = data.current_path.get();
    let size = human_readable_number(data.total_file_size.load(ORDERING), output_display);
    format!("Preparing: {path_in} {size} ... {prog_char}")
}

fn format_indexing_str(prog_char: char, data: &PAtomicInfo, output_display: &str) -> String {
    let path_in = data.current_path.get();
    let file_count = data.num_files.load(ORDERING);
    let size = human_readable_number(data.total_file_size.load(ORDERING), output_display);
    let file_str = format!("{file_count} files, {size}");
    format!("Indexing: {path_in} {file_str} ... {prog_char}")
}

pub struct PIndicator {
    pub thread: Option<(Sender<()>, JoinHandle<()>)>,
    pub data: Arc<PAtomicInfo>,
}

impl PIndicator {
    pub fn build_me() -> Self {
        Self {
            thread: None,
            data: Arc::new(PAtomicInfo {
                ..Default::default()
            }),
        }
    }

    pub fn spawn(&mut self, output_display: String) {
        let data = self.data.clone();
        let (stop_handler, receiver) = mpsc::channel::<()>();

        let time_info_thread = std::thread::spawn(move || {
            let mut progress_char_i: usize = 0;
            let mut stdout = std::io::stdout();
            let mut msg = "".to_string();

            // While the timeout triggers we go round the loop
            // If we disconnect or the sender sends its message we exit the while loop
            while let Err(RecvTimeoutError::Timeout) =
                receiver.recv_timeout(Duration::from_millis(SPINNER_SLEEP_TIME))
            {
                // Clear the text written by 'write!'& Return at the start of line
                print!("\r{:width$}", " ", width = msg.len());
                let prog_char = PROGRESS_CHARS[progress_char_i];

                msg = match data.state.load(ORDERING) {
                    Operation::INDEXING => format_indexing_str(prog_char, &data, &output_display),
                    Operation::PREPARING => format_preparing_str(prog_char, &data, &output_display),
                    _ => panic!("Unknown State"),
                };

                write!(stdout, "\r{msg}").unwrap();
                stdout.flush().unwrap();

                progress_char_i += 1;
                progress_char_i %= PROGRESS_CHARS_LEN;
            }
            print!("\r{:width$}", " ", width = msg.len());
            print!("\r");
            stdout.flush().unwrap();
        });
        self.thread = Some((stop_handler, time_info_thread))
    }

    pub fn stop(self) {
        if let Some((stop_handler, thread)) = self.thread {
            stop_handler.send(()).unwrap();
            thread.join().unwrap();
        }
    }
}
