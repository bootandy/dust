use std::{
    fmt::Display,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::{Duration},
};

use crate::display;

/* -------------------------------------------------------------------------- */

pub const ATOMIC_ORDERING: Ordering = Ordering::Relaxed;

const PROGRESS_CHARS_DELTA: u64 = 100;
const PROGRESS_CHARS: [char; 4] = ['-', '\\', '|', '/'];
const PROGRESS_CHARS_LEN: usize = PROGRESS_CHARS.len();

// small wrappers for atomic number to reduce overhead
pub trait ThreadSyncTrait<T> {
    fn set(&self, val: T);
    fn get(&self) -> T;
}

pub trait ThreadSyncMathTrait<T> {
    fn add(&self, val: T);
}

macro_rules! create_atomic_wrapper {
    ($ident: ident, $atomic_type: ty, $type: ty, $ordering: ident) => {
        #[derive(Default)]
        pub struct $ident {
            inner: $atomic_type,
        }

        impl ThreadSyncTrait<$type> for $ident {
            fn set(&self, val: $type) {
                self.inner.store(val, $ordering)
            }

            fn get(&self) -> $type {
                self.inner.load($ordering)
            }
        }
    };

    ($ident: ident, $atomic_type: ty, $type: ty, $ordering: ident + add) => {
        create_atomic_wrapper!($ident, $atomic_type, $type, $ordering);

        impl ThreadSyncMathTrait<$type> for $ident {
            fn add(&self, val: $type) {
                self.inner.fetch_add(val, $ordering);
            }
        }
    };
}

create_atomic_wrapper!(AtomicU64Wrapper, AtomicU64, u64, ATOMIC_ORDERING + add);
create_atomic_wrapper!(AtomicU8Wrapper, AtomicU8, u8, ATOMIC_ORDERING + add);

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
    pub file_number: AtomicU64Wrapper,
    pub total_file_size: TotalSize,
    pub state: AtomicU8Wrapper,
    pub current_path: ThreadStringWrapper,
}

impl PAtomicInfo {
    fn new(c: &PConfig) -> Self {
        Self {
            total_file_size: TotalSize::new(c),
            ..Default::default()
        }
    }
}

/* -------------------------------------------------------------------------- */

#[derive(Default)]
pub struct TotalSize {
    use_iso: bool,
    inner: AtomicU64Wrapper,
}

impl TotalSize {
    fn new(c: &PConfig) -> Self {
        Self {
            use_iso: c.use_iso,
            ..Default::default()
        }
    }
}

impl Display for TotalSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&display::human_readable_number(
            self.inner.get(),
            self.use_iso,
        ))
    }
}

impl ThreadSyncTrait<u64> for TotalSize {
    fn set(&self, val: u64) {
        self.inner.set(val)
    }

    fn get(&self) -> u64 {
        self.inner.get()
    }
}

impl ThreadSyncMathTrait<u64> for TotalSize {
    fn add(&self, val: u64) {
        self.inner.add(val)
    }
}

/* -------------------------------------------------------------------------- */
fn format_indicator_str(data: &PAtomicInfo, progress_char_i: usize, s: &str) -> String {
    format!(
        "\r{} \"{}\"... {}",
        s,
        data.current_path.get(),
        PROGRESS_CHARS[progress_char_i],
    )
}

#[derive(Default)]
pub struct PConfig {
    pub use_iso: bool,
}

pub struct PIndicator {
    thread_run: Arc<AtomicBool>,
    pub thread: Option<JoinHandle<()>>,
    pub data: Arc<PAtomicInfo>,
    pub config: Arc<PConfig>,
}

impl PIndicator {
    pub fn build_me(c: PConfig) -> Self {
        Self {
            thread_run: Arc::new(AtomicBool::new(true)),
            thread: None,
            data: Arc::new(PAtomicInfo::new(&c)),
            config: Arc::new(c),
        }
    }

    pub fn spawn(&mut self) {
        let data = self.data.clone();
        let is_building_data_const = self.thread_run.clone();

        let time_info_thread = std::thread::spawn(move || {
            let mut progress_char_i: usize = 0;
            let mut stdout = std::io::stdout();
            std::thread::sleep(Duration::from_millis(PROGRESS_CHARS_DELTA));

            while is_building_data_const.load(ATOMIC_ORDERING) {
                let msg = match data.state.get() {
                    Operation::INDEXING => {
                        let base = format_indicator_str(&data, progress_char_i, "Indexing");

                        let file_count = data.file_number.get();
                        let file_str =
                            format!("{} {} files", file_count, data.total_file_size);

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

                std::thread::sleep(Duration::from_millis(PROGRESS_CHARS_DELTA));
                // Clear the text written by 'write!'
                print!("\r{:width$}", " ", width = msg.len());
            }

            // Return at the start of the line so the output can be printed correctly
            print!("\r");
            stdout.flush().unwrap();
        });
        self.thread = Some(time_info_thread)
    }

    pub fn stop(self) {
        self.thread_run.store(false, ATOMIC_ORDERING);
        if let Some(t) = self.thread {
            t.join().unwrap();
        }
    }
}
