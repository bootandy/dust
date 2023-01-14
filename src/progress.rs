use std::{
    fmt::Display,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::display;

/* -------------------------------------------------------------------------- */

pub const ATOMIC_ORDERING: Ordering = Ordering::Relaxed;

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

#[derive(Default)]
pub struct PConfig {
    pub file_count_only: bool,
    pub ignore_hidden: bool,
    pub use_iso: bool,
}

pub struct PIndicator {
    thread_run: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    pub data: Arc<PAtomicInfo>,
    pub config: Arc<PConfig>,
}

impl PIndicator {
    pub fn spawn(config: PConfig) -> Self {
        macro_rules! init_shared_data {
            (let $ident: ident, $ident2: ident = $value: expr) => {
                let $ident = Arc::new($value);
                let $ident2 = $ident.clone();
            };
        }

        init_shared_data!(let instant, instant2 = Instant::now());
        init_shared_data!(let time_thread_run, time_thread_run2 = AtomicBool::new(true));
        init_shared_data!(let config, config2 = config);
        init_shared_data!(let data, data2 = PAtomicInfo::new(&config));

        let time_info_thread = std::thread::spawn(move || {
            const SHOW_WALKING_AFTER: u64 = 0;

            const PROGRESS_CHARS_DELTA: u64 = 100;
            const PROGRESS_CHARS: [char; 4] = ['-', '\\', '|', '/'];
            const PROGRESS_CHARS_LEN: usize = PROGRESS_CHARS.len();
            let mut progress_char_i: usize = 0;

            let mut stdout = std::io::stdout();

            let mut last_msg_len = 0;

            while time_thread_run2.load(ATOMIC_ORDERING) {
                if instant2.elapsed() > Duration::from_secs(SHOW_WALKING_AFTER) {
                    // print!("{:?}", *state2.read().unwrap());

                    // clear the line
                    print!("\r{:width$}", " ", width = last_msg_len);

                    macro_rules! format_base {
                        ($state: expr) => {
                            format!(
                                "\r{} \"{}\"... {}",
                                $state,
                                data2.current_path.get(),
                                PROGRESS_CHARS[progress_char_i],
                            )
                        };
                    }

                    let msg = match data2.state.get() {
                        Operation::INDEXING => {
                            const PROPS_SEPARATOR: &str = ", ";

                            let base = format_base!("Indexing");

                            macro_rules! format_property {
                                ($value: ident, $singular: expr, $plural: expr) => {
                                    format!(
                                        "{} {}",
                                        $value,
                                        if $value > 1 { $plural } else { $singular }
                                    )
                                };
                            }

                            let mut main_props = Vec::new();

                            let fn_ = data2.file_number.get();
                            if config2.file_count_only {
                                main_props.push(format_property!(fn_, "file", "files"));
                            } else {
                                main_props.push(format!("{}", data2.total_file_size));
                                main_props.push(format_property!(fn_, "file", "files"));
                            };

                            let main_props_str = main_props.join(PROPS_SEPARATOR);
                            format!("{} - {}", base, main_props_str)
                        }
                        Operation::PREPARING => {
                            format_base!("Preparing")
                        }
                        _ => panic!("Unknown State"),
                    };
                    last_msg_len = msg.len();

                    write!(stdout, "{}", msg).unwrap();
                    stdout.flush().unwrap();

                    progress_char_i += 1;
                    progress_char_i %= PROGRESS_CHARS_LEN;

                    std::thread::sleep(Duration::from_millis(PROGRESS_CHARS_DELTA));
                } else {
                    // wait duration is in seconds so we need only to check each second
                    std::thread::sleep(Duration::from_secs(1));
                }
            }

            // clear the line for the last time
            print!("\r{:width$}", " ", width = last_msg_len);

            // Return at the start of the line so the output can be printed correctly
            print!("\r");
            stdout.flush().unwrap();
        });

        Self {
            thread_run: time_thread_run,
            thread: time_info_thread,
            data,
            config,
        }
    }

    pub fn stop(self) {
        self.thread_run.store(false, ATOMIC_ORDERING);
        self.thread.join().unwrap();
    }
}
