use std::{
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::Duration,
};

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
    pub no_permissions: AtomicBool,
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

/* -------------------------------------------------------------------------- */
fn format_indicator_str(data: &PAtomicInfo, progress_char_i: usize, status: &str) -> String {
    format!(
        "\r{} \"{}\"... {}",
        status,
        data.current_path.get(),
        PROGRESS_CHARS[progress_char_i],
    )
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

    pub fn spawn(&mut self, is_iso: bool) {
        let data = self.data.clone();
        let (stop_handler, receiver) = mpsc::channel::<()>();

        let time_info_thread = std::thread::spawn(move || {
            let mut progress_char_i: usize = 0;
            let mut stdout = std::io::stdout();

            // While the timeout triggers we go round the loop
            // If we disconnect or the sender sends its message we exit the while loop
            while let Err(RecvTimeoutError::Timeout) =
                receiver.recv_timeout(Duration::from_millis(SPINNER_SLEEP_TIME))
            {
                let msg = match data.state.load(ORDERING) {
                    Operation::INDEXING => {
                        let base = format_indicator_str(&data, progress_char_i, "Indexing");

                        let file_count = data.num_files.load(ORDERING);
                        let size =
                            human_readable_number(data.total_file_size.load(ORDERING), is_iso);
                        let file_str = format!("{} {} files", file_count, size);
                        format!("{} - {}", base, file_str)
                    }
                    Operation::PREPARING => {
                        format_indicator_str(&data, progress_char_i, "Preparing")
                    }
                    _ => panic!("Unknown State"),
                };

                write!(stdout, "{}", msg).unwrap();
                stdout.flush().unwrap();

                progress_char_i += 1;
                progress_char_i %= PROGRESS_CHARS_LEN;

                // Clear the text written by 'write!'
                print!("\r{:width$}", " ", width = msg.len());
            }

            // Return at the start of the line so the output can be printed correctly
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
