use std::{
    fmt::Display,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use clap::ArgMatches;

use crate::{config::Config, dir_walker::WalkData};

pub const ATOMIC_ORDERING: Ordering = Ordering::Relaxed;

#[macro_export]
macro_rules! init_shared_data {
    (let $ident: ident, $ident2: ident = $value: expr) => {
        let $ident = Arc::new($value);
        let $ident2 = $ident.clone();
    };
}

/* -------------------------------------------------------------------------- */

// a small wrapper for atomic number to reduce overhead
pub trait AtomicWrapperTrait<T> {
    fn set(&self, val: T);
    fn add(&self, val: T);
    fn get(&self) -> T;
}

macro_rules! create_atomic_wrapper {
    ($ident: ident, $atomic_type: ty, $type: ty, $ordering: ident) => {
        #[derive(Default)]
        pub struct $ident {
            inner: $atomic_type,
        }

        impl AtomicWrapperTrait<$type> for $ident {
            fn set(&self, val: $type) {
                self.inner.store(val, $ordering)
            }

            fn add(&self, val: $type) {
                self.inner.fetch_add(val, $ordering);
            }

            fn get(&self) -> $type {
                self.inner.load($ordering)
            }
        }
    };
}

create_atomic_wrapper!(AtomicU64Wrapper, AtomicU64, u64, ATOMIC_ORDERING);
create_atomic_wrapper!(AtomicU8Wrapper, AtomicU8, u8, ATOMIC_ORDERING);

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
    pub files_skipped: AtomicU64Wrapper,
    pub directories_skipped: AtomicU64Wrapper,
    pub total_file_size: TotalSize,
    pub state: AtomicU8Wrapper,
}

impl PAtomicInfo {
    fn new(c: &PConfig) -> Self {
        Self {
            total_file_size: TotalSize::new(c),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct TotalSize {
    use_iso: bool,
    pub inner: AtomicU64Wrapper,
}

impl TotalSize {
    fn new(c: &PConfig) -> Self {
        Self {
            use_iso: c.use_iso,
            ..Default::default()
        }
    }

    fn format_size(&self) -> String {
        let inner = self.inner.get();
        let number_len = (inner as f32).log10().floor() as u32;

        let end = self.get_size_end(number_len);

        let size_base: u64 = if self.use_iso { 1000 } else { 1024 };
        let showed_number = inner / (size_base.pow((number_len / 3).min(4)));
        format!("{} {}", showed_number, end)
        // format!("{} bytes", inner)
    }

    fn get_size_end(&self, size: u32) -> &'static str {
        match size / 3 {
            0 => "bytes",
            1 => "K",
            2 => "M",
            3 => "G",
            _ => "T",
        }
    }
}

impl Display for TotalSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_size().as_str())
    }
}

/* -------------------------------------------------------------------------- */

#[derive(Default)]
pub struct PConfig {
    pub file_count_only: bool,
    use_iso: bool,
}

impl From<(&'_ WalkData<'_>, &'_ Config, &'_ ArgMatches)> for PConfig {
    fn from(in_: (&WalkData, &Config, &ArgMatches)) -> Self {
        let w = in_.0;
        let c = in_.1;
        let o = in_.2;

        Self {
            file_count_only: w.by_filecount,
            use_iso: c.get_iso(o),
        }
    }
}

pub struct PIndicator {
    thread_run: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    pub data: Arc<PAtomicInfo>,
    pub config: Arc<PConfig>,
}

impl PIndicator {
    pub fn spawn(walk_config: &WalkData, config: &Config, args: &ArgMatches) -> Self {
        init_shared_data!(let instant, instant2 = Instant::now());
        init_shared_data!(let time_thread_run, time_thread_run2 = AtomicBool::new(true));
        init_shared_data!(let config, config2 = PConfig::from((walk_config, config, args)));
        init_shared_data!(let data, data2 = PAtomicInfo::new(&config));

        let time_info_thread = std::thread::spawn(move || {
            const SHOW_WALKING_AFTER: u64 = 2;

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

                    let msg = match data2.state.get() {
                        Operation::INDEXING => {
                            let base =
                                format!("\rIndexing... {}", PROGRESS_CHARS[progress_char_i],);

                            let base = if config2.file_count_only {
                                format!("{} - {} files", base, data2.file_number.get())
                            } else {
                                format!(
                                    "{} - {} ({} files)",
                                    base,
                                    data2.total_file_size,
                                    data2.file_number.get()
                                )
                            };

                            let ds = data2.directories_skipped.get();
                            let fs = data2.files_skipped.get();

                            macro_rules! format_property {
                                ($value: ident, $singular: expr, $plural: expr) => {
                                    format!(
                                        "{} {}",
                                        $value,
                                        if $value > 1 { $plural } else { $singular }
                                    )
                                };
                            }

                            if ds + fs != 0 {
                                let mut strs = Vec::new();
                                if fs != 0 {
                                    strs.push(format_property!(fs, "file", "files"))
                                }

                                if ds != 0 {
                                    strs.push(format_property!(ds, "directory", "directories"))
                                }

                                format!("{} ({} skipped)", base, strs.join(", "))
                            } else {
                                base
                            }
                        }
                        Operation::PREPARING => {
                            format!("\rPreparing... {}", PROGRESS_CHARS[progress_char_i],)
                        }
                        _ => panic!("Unknown State"),
                    };
                    last_msg_len = msg.len();

                    print!("{}", msg);

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
