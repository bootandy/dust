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

pub const ATOMIC_ORDERING: Ordering = Ordering::SeqCst;

#[macro_export]
macro_rules! init_shared_data {
    (let $ident: ident, $ident2: ident = $value: expr) => {
        let $ident = Arc::new($value);
        let $ident2 = $ident.clone();
    };
}

/* -------------------------------------------------------------------------- */

// it's easier to create an "enum" this way because of the atomic loading
#[allow(non_snake_case)]
pub mod State {
    pub const WALKING: u8 = 0;
    pub const CLEANING: u8 = 1;
}

#[derive(Default)]
pub struct AtomicInfoData {
    pub file_number: AtomicU64,
    pub total_file_size: TotalSize,
    pub state: AtomicU8,
}

#[derive(Default)]
pub struct TotalSize {
    pub inner: AtomicU64,
}

impl TotalSize {
    fn format_size(&self) -> String {
        let inner = self.inner.load(ATOMIC_ORDERING);
        let number_len = (inner as f32).log10().floor() as u32;

        let end = self.get_size_end(number_len);

        let showed_number = inner / (10_u64.pow(number_len / 3 * 3));
        format!("{} {}", showed_number, end)
    }

    fn get_size_end(&self, size: u32) -> &'static str {
        match size / 3 {
            0 => "bytes",
            1 => "Kb",
            2 => "Mb",
            3 => "Gb",
            _ => "Tb",
        }
    }
}

impl Display for TotalSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_size().as_str())
    }
}

impl AtomicInfoData {
    fn get_data(&self) -> InfoData {
        InfoData {
            file_number: self.file_number.load(ATOMIC_ORDERING),
            total_file_size: self.total_file_size.inner.load(ATOMIC_ORDERING),
        }
    }
}

pub struct InfoData {
    pub file_number: u64,
    pub total_file_size: u64,
}

/* -------------------------------------------------------------------------- */

pub struct Info {
    thread_run: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    pub data: Arc<AtomicInfoData>,
}

impl Info {
    pub fn spawn() -> Self {
        init_shared_data!(let instant, instant2 = Instant::now());
        init_shared_data!(let time_thread_run, time_thread_run2 = AtomicBool::new(true));
        init_shared_data!(let data, data2 = AtomicInfoData::default());

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

                    let msg = match data2.state.load(ATOMIC_ORDERING) {
                        State::WALKING => format!(
                            "\rIndexing... {} - {} ({} files)",
                            PROGRESS_CHARS[progress_char_i],
                            data2.total_file_size,
                            data2.file_number.load(ATOMIC_ORDERING)
                        ),
                        State::CLEANING => {
                            format!("\rPreparing... {}", PROGRESS_CHARS[progress_char_i],)
                        }
                        _ => panic!("Unknown State"),
                    };
                    last_msg_len = msg.len();

                    print!("{}", msg);

                    stdout.flush().unwrap();

                    progress_char_i += 1;
                    progress_char_i %= PROGRESS_CHARS_LEN;

                    std::thread::sleep(Duration::from_millis(PROGRESS_CHARS_DELTA))
                }
            }

            // clear the walking line before showing the result
            // TODO : test if this works on windows
            print!("\r");
            stdout.flush().unwrap();
        });

        Self {
            thread_run: time_thread_run,
            thread: time_info_thread,
            data: data,
        }
    }

    pub fn stop(self) -> InfoData {
        self.thread_run.store(false, ATOMIC_ORDERING);
        self.thread.join().unwrap();

        self.data.get_data()
    }
}
