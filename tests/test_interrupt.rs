#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::os::unix::fs::symlink;
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const SCAN_DEPTH: u32 = 10;
const BRANCHES: u32 = 4;

// Ensure a failed assertion cannot leave a scan running in the background.
struct RunningDust(Child);

impl Drop for RunningDust {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn interrupt_scan(args: &[&str]) -> (Output, u64) {
    let fixture = tempfile::tempdir().unwrap();
    let mut root = fixture.path().join("level0");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("sample.bin"), b"already scanned").unwrap();

    // A finite tree of symlinks provides a long scan with only a few real files.
    // Wait for actual progress before signaling, rather than guessing a delay.
    for depth in 1..=SCAN_DEPTH {
        let next = fixture.path().join(if depth == SCAN_DEPTH {
            "scan".to_string()
        } else {
            format!("level{depth}")
        });
        fs::create_dir(&next).unwrap();
        fs::write(next.join("sample.bin"), b"already scanned").unwrap();
        for branch in 0..BRANCHES {
            symlink(&root, next.join(format!("branch{branch}"))).unwrap();
        }
        root = next;
    }

    let mut dust = RunningDust(
        Command::new(cargo_bin!("dust"))
            .args(["-L", "-s", "-T", "2"])
            .args(args)
            .arg(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );

    let mut stdout = dust.0.stdout.take().unwrap();
    let stdout_thread = thread::spawn(move || {
        let mut output = Vec::new();
        stdout.read_to_end(&mut output).unwrap();
        output
    });
    let mut stderr = BufReader::new(dust.0.stderr.take().unwrap());
    let (progress_tx, progress_rx) = mpsc::channel();
    let stderr_thread = thread::spawn(move || {
        let mut output = Vec::new();
        let mut line = Vec::new();
        let mut reported_progress = false;
        while stderr.read_until(b'\r', &mut line).unwrap() > 0 {
            if !reported_progress {
                let text = String::from_utf8_lossy(&line);
                if let Some((prefix, _)) = text.split_once(" files,")
                    && let Some(count) = prefix.split_whitespace().last()
                    && let Ok(count) = count.parse::<u64>()
                    && count > 0
                {
                    progress_tx.send(count).unwrap();
                    reported_progress = true;
                }
            }
            output.extend_from_slice(&line);
            line.clear();
        }
        output
    });

    let scanned_files = progress_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("scan did not report progress");
    assert!(
        Command::new("kill")
            .args(["-INT", &dust.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = dust.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "scan did not stop after Ctrl-C");
        thread::sleep(Duration::from_millis(10));
    };

    (
        Output {
            status,
            stdout: stdout_thread.join().unwrap(),
            stderr: stderr_thread.join().unwrap(),
        },
        scanned_files,
    )
}

#[test]
fn test_ctrl_c_prints_partial_tree() {
    let (output, _) = interrupt_scan(&["-c", "-b", "-n", "30"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(130));
    assert!(stdout.starts_with("Partial result: scan interrupted by Ctrl-C.\n"));
    assert!(stdout.contains("┌─┴ scan"), "{stdout}");
    assert!(stdout.contains("branch"), "{stdout}");
    assert!(!stdout.contains('\r'));
}

#[test]
fn test_ctrl_c_json_retains_collected_files_and_totals() {
    let (output, scanned_files) = interrupt_scan(&["-j", "-f"]);
    assert_eq!(output.status.code(), Some(130));

    let tree: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tree["partial"], true);
    assert!(tree["name"].as_str().unwrap().ends_with("/scan"));

    fn check_totals(node: &serde_json::Value) -> u64 {
        let size = node["size"].as_str().unwrap().parse::<u64>().unwrap();
        let children = node["children"].as_array().unwrap();
        if node["name"].as_str().unwrap().ends_with("/sample.bin") {
            assert_eq!(size, 1);
            assert!(children.is_empty());
        } else {
            assert_eq!(size, children.iter().map(check_totals).sum::<u64>());
        }
        size
    }

    let count = check_totals(&tree);
    let full_count: u64 = (0..=SCAN_DEPTH)
        .map(|depth| u64::from(BRANCHES).pow(depth))
        .sum();
    assert!(count >= scanned_files, "already counted files were lost");
    assert!(
        count < full_count,
        "scan completed instead of stopping early"
    );
}
