use std::{ffi::OsStr, future::Future, panic, path::Path, process::Command, sync::OnceLock};

use regex::Regex;
use tokio::task::JoinSet;

use db::model::blob::FileType;

static COLLAPSE_WHITESPACE: OnceLock<Regex> = OnceLock::new();

/// Returns a human-friendly file name (no path); strips extension if not a directory.
///
/// # Panics
///
/// Panics if the file name is not valid UTF-8.
#[must_use]
pub fn prettify_file_name(file: &Path, is_dir: bool) -> String {
    let extension = file.extension();
    let mut file_name: String = String::from(
        file.file_name()
            .unwrap_or_else(|| OsStr::new("unknown_filename"))
            .to_str()
            .unwrap(),
    );

    if !is_dir && let Some(ext) = extension {
        file_name = String::from(&file_name[0..file_name.len() - ext.len() - 1]);
    }

    let remove_whitespace = COLLAPSE_WHITESPACE.get_or_init(|| Regex::new(r" {2,}").unwrap());

    file_name = file_name.replace(['_', '-', '+'], " ");

    file_name = String::from(remove_whitespace.replace_all(&file_name, " "));

    file_name = String::from(file_name.trim());

    file_name
}

#[must_use]
pub fn cleanse_evil_from_name(name: &str) -> String {
    let cleansed = name
        .replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], " ")
        .trim()
        .to_string();

    // "." or ".." contain no separator to strip above, but `Path::join` still
    // treats them as directory traversal (escaping into the parent dir).
    if !cleansed.is_empty() && cleansed.chars().all(|c| c == '.') {
        return String::new();
    }

    cleansed
}

/// Opens the given path in the system file explorer.
///
/// # Panics
///
/// Panics if the path is not valid UTF-8.
pub fn open_folder_in_explorer(path: &Path) {
    let path = path.to_str().unwrap();
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).output().unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).output().unwrap();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(path).output().unwrap();
    }
}

/// Returns the total size of all files in the directory (non-recursive).
///
/// # Panics
///
/// Panics if the directory cannot be read or a metadata call fails.
#[must_use]
pub fn get_folder_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .unwrap()
        .map(|f| f.unwrap().metadata().unwrap().len())
        .sum()
}

#[must_use]
pub fn is_zippable_file_extension(extension: &str) -> bool {
    FileType::from_extension(extension).is_zippable()
}

#[must_use]
pub fn is_zipped_file_extension(extension: &str) -> bool {
    FileType::from_extension(extension).is_zipped()
}

#[must_use]
pub fn convert_extension_to_zip(extension: &str) -> String {
    let lowercase = extension.to_lowercase();

    String::from(match lowercase.as_str() {
        "stl" => "stl.zip",
        "obj" => "obj.zip",
        "step" => "step.zip",
        "gcode" => "gcode.zip",
        _ => &lowercase,
    })
}

#[must_use]
pub fn convert_zip_to_extension(extension: &str) -> String {
    let lowercase = extension.to_lowercase();

    String::from(match lowercase.as_str() {
        "stl.zip" => "stl",
        "obj.zip" => "obj",
        "step.zip" => "step",
        "gcode.zip" => "gcode",
        _ => &lowercase,
    })
}

/// Runs one future per item with bounded parallelism, returning every task output in completion order.
///
/// Spawns the future produced by `make_future` for each item, keeping at most `max` tasks in
/// flight (throttling by awaiting a completed task once the in-flight count reaches `max`).
/// Outputs are collected in the order tasks complete, matching `JoinSet::join_all`.
///
/// # Panics
///
/// Propagates a panicking task exactly as `JoinSet` does: a panicked task resumes the unwind
/// (`panic::resume_unwind`), and any other `JoinError` panics.
pub async fn run_bounded<Item, Fut, F, T>(
    mut items: Vec<Item>,
    max: usize,
    mut make_future: F,
) -> Vec<T>
where
    F: FnMut(Item) -> Fut,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mut futures: JoinSet<T> = JoinSet::new();
    let mut results: Vec<T> = Vec::new();

    while let Some(item) = items.pop() {
        futures.spawn(make_future(item));

        if futures.len() >= max
            && let Some(res) = futures.join_next().await
        {
            match res {
                Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                Err(err) => panic!("{err}"),
                Ok(output) => {
                    results.push(output);
                }
            }
        }
    }

    results.extend(futures.join_all().await);

    results
}

// -----------------------------------------------------------------------------
// Regression tests for run_bounded: lock in the current behaviour shared by the
// import and export JoinSet loops before/after the dedup refactor — every task
// output is returned, at most `max` tasks run concurrently, and a panicking task
// propagates as a panic (resume_unwind).
// -----------------------------------------------------------------------------

#[cfg(test)]
mod run_bounded_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::run_bounded;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn returns_every_task_output() {
        let items: Vec<usize> = (0..20).collect();

        let mut results = run_bounded(items.clone(), 4, |item| async move { item * 2 }).await;

        // Completion order is nondeterministic, so compare as a sorted multiset.
        results.sort_unstable();
        let mut expected: Vec<usize> = items.into_iter().map(|item| item * 2).collect();
        expected.sort_unstable();

        assert_eq!(results, expected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn empty_input_returns_empty_vec() {
        let results = run_bounded(Vec::<usize>::new(), 4, |item| async move { item }).await;

        assert!(results.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn never_exceeds_max_in_flight() {
        let max = 3;
        let in_flight = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let items: Vec<usize> = (0..30).collect();

        run_bounded(items, max, |item| {
            let in_flight = Arc::clone(&in_flight);
            let peak = Arc::clone(&peak);

            async move {
                let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(current, Ordering::SeqCst);

                // Yield repeatedly so other tasks can be scheduled while this one is "in flight".
                for _ in 0..8 {
                    tokio::task::yield_now().await;
                }

                in_flight.fetch_sub(1, Ordering::SeqCst);

                item
            }
        })
        .await;

        assert!(
            peak.load(Ordering::SeqCst) <= max,
            "peak in-flight {} exceeded max {max}",
            peak.load(Ordering::SeqCst)
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[should_panic(expected = "task boom")]
    async fn propagates_task_panic() {
        let items: Vec<usize> = (0..4).collect();

        // A panicking task must surface as a panic via panic::resume_unwind.
        run_bounded(items, 2, |_item| async move { panic!("task boom") }).await;
    }
}

// -----------------------------------------------------------------------------
// Regression tests for service::util pure helpers.
// We test these after consolidating util so that src-tauri uses service::util;
// the goal is to lock in behaviour and catch regressions from deduplication.
// Only pure functions are unit-tested here; IO (get_folder_size,
// open_folder_in_explorer) is left out unless tested via temp dir.
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::prelude::*;

    use super::{
        cleanse_evil_from_name, convert_extension_to_zip, convert_zip_to_extension,
        is_zippable_file_extension, is_zipped_file_extension, prettify_file_name,
    };

    // ---- cleanse_evil_from_name ----

    #[test]
    fn test_cleanse_evil_empty() {
        assert_eq!(cleanse_evil_from_name(""), "");
    }

    #[test]
    fn test_cleanse_evil_normal_unchanged() {
        assert_eq!(cleanse_evil_from_name("hello world"), "hello world");
    }

    #[test]
    fn test_cleanse_evil_removes_backslash() {
        assert_eq!(cleanse_evil_from_name("a\\b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_slash() {
        assert_eq!(cleanse_evil_from_name("a/b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_colon() {
        assert_eq!(cleanse_evil_from_name("a:b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_asterisk() {
        assert_eq!(cleanse_evil_from_name("a*b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_question_mark() {
        assert_eq!(cleanse_evil_from_name("a?b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_double_quote() {
        assert_eq!(cleanse_evil_from_name("a\"b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_less_than() {
        assert_eq!(cleanse_evil_from_name("a<b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_greater_than() {
        assert_eq!(cleanse_evil_from_name("a>b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_removes_pipe() {
        assert_eq!(cleanse_evil_from_name("a|b"), "a b");
    }

    #[test]
    fn test_cleanse_evil_trim_outer_whitespace() {
        assert_eq!(cleanse_evil_from_name("  x  "), "x");
    }

    #[test]
    fn test_cleanse_evil_rejects_dot_only_names() {
        // No separators to strip, but ".." still escapes `Path::join` into the parent dir.
        assert_eq!(cleanse_evil_from_name(".."), "");
        assert_eq!(cleanse_evil_from_name("."), "");
        assert_eq!(cleanse_evil_from_name("..."), "");
    }

    // ---- prettify_file_name ----

    #[test]
    fn test_prettify_file_name_path_with_extension_is_file() {
        let path = PathBuf::from("some_dir/My_Cool-Model.stl");
        assert_eq!(prettify_file_name(&path, false), "My Cool Model");
    }

    #[test]
    fn test_prettify_file_name_path_with_extension_is_dir() {
        let path = PathBuf::from("some_dir/My_Folder.stl");
        assert_eq!(prettify_file_name(&path, true), "My Folder.stl");
    }

    #[test]
    fn test_prettify_file_name_no_extension() {
        let path = PathBuf::from("myfile");
        assert_eq!(prettify_file_name(&path, false), "myfile");
    }

    #[test]
    fn test_prettify_file_name_replaces_underscore_and_collapses_spaces() {
        let path = PathBuf::from("a__b___c.stl");
        assert_eq!(prettify_file_name(&path, false), "a b c");
    }

    // ---- is_zippable_file_extension ----

    #[test]
    fn test_is_zippable_stl() {
        assert!(is_zippable_file_extension("stl"));
    }

    #[test]
    fn test_is_zippable_stl_uppercase() {
        assert!(is_zippable_file_extension("STL"));
    }

    #[test]
    fn test_is_zippable_obj() {
        assert!(is_zippable_file_extension("obj"));
    }

    #[test]
    fn test_is_zippable_step() {
        assert!(is_zippable_file_extension("step"));
    }

    #[test]
    fn test_is_zippable_gcode() {
        assert!(is_zippable_file_extension("gcode"));
    }

    #[test]
    fn test_is_zippable_txt_returns_false() {
        assert!(!is_zippable_file_extension("txt"));
    }

    /// db maps "stp" to `Step`; `FileType` delegation covers this.
    #[test]
    fn test_is_zippable_stp() {
        assert!(is_zippable_file_extension("stp"));
    }

    #[test]
    fn test_is_zippable_step_uppercase() {
        assert!(is_zippable_file_extension("STEP"));
    }

    // ---- is_zipped_file_extension ----

    #[test]
    fn test_is_zipped_stl_zip() {
        assert!(is_zipped_file_extension("stl.zip"));
    }

    #[test]
    fn test_is_zipped_step_zip() {
        assert!(is_zipped_file_extension("step.zip"));
    }

    #[test]
    fn test_is_zipped_obj_zip_gcode_zip() {
        assert!(is_zipped_file_extension("obj.zip"));
        assert!(is_zipped_file_extension("gcode.zip"));
    }

    #[test]
    fn test_is_zipped_other_returns_false() {
        assert!(!is_zipped_file_extension("stl"));
        assert!(!is_zipped_file_extension("zip"));
        assert!(!is_zipped_file_extension("obj"));
    }

    // ---- convert_extension_to_zip ----

    #[test]
    fn test_convert_extension_to_zip_stl() {
        assert_eq!(convert_extension_to_zip("stl"), "stl.zip");
    }

    #[test]
    fn test_convert_extension_to_zip_obj_step_gcode() {
        assert_eq!(convert_extension_to_zip("obj"), "obj.zip");
        assert_eq!(convert_extension_to_zip("step"), "step.zip");
        assert_eq!(convert_extension_to_zip("gcode"), "gcode.zip");
    }

    #[test]
    fn test_convert_extension_to_zip_unknown_returns_lowercase() {
        assert_eq!(convert_extension_to_zip("TXT"), "txt");
        assert_eq!(convert_extension_to_zip("unknown"), "unknown");
    }

    // ---- convert_zip_to_extension ----

    #[test]
    fn test_convert_zip_to_extension_known() {
        assert_eq!(convert_zip_to_extension("stl.zip"), "stl");
        assert_eq!(convert_zip_to_extension("obj.zip"), "obj");
        assert_eq!(convert_zip_to_extension("step.zip"), "step");
        assert_eq!(convert_zip_to_extension("gcode.zip"), "gcode");
    }

    #[test]
    fn test_convert_zip_to_extension_unknown_returns_lowercase() {
        assert_eq!(convert_zip_to_extension("other.zip"), "other.zip");
        assert_eq!(convert_zip_to_extension("STL.ZIP"), "stl");
    }

    // ---- proptest: round-trip and invariants ----

    const EVIL_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

    proptest! {
        /// Round-trip: for known zippable extensions, convert_zip_to_extension(convert_extension_to_zip(ext)) equals canonical form.
        #[test]
        fn prop_convert_extension_round_trip_known(ext in prop_oneof![
            Just("stl"),
            Just("obj"),
            Just("gcode"),
            Just("step"),
            Just("stp"),
        ]) {
            let z = convert_extension_to_zip(ext);
            let back = convert_zip_to_extension(&z);
            prop_assert_eq!(back.as_str(), ext);
        }

        /// For any string, cleanse_evil output must not contain filesystem-unsafe characters.
        #[test]
        fn prop_cleanse_evil_removes_evil(s in "[ -~]{0,80}") {
            let out = cleanse_evil_from_name(&s);
            for c in EVIL_CHARS {
                prop_assert!(!out.contains(*c), "output must not contain {:?}", c);
            }
        }

        /// Idempotent: cleansing an already-clean string only trims.
        #[test]
        fn prop_cleanse_evil_idempotent_clean(s in "[a-zA-Z0-9 _-]{0,50}") {
            let trimmed = s.trim();
            let once = cleanse_evil_from_name(trimmed);
            let twice = cleanse_evil_from_name(&once);
            prop_assert_eq!(once, twice);
        }
    }
}
