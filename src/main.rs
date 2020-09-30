use anyhow::Result;
use chrono::TimeZone;
use chrono::Utc;
use regex::Regex;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

struct Note {
    path: String,
    line: usize,
    timestamp: i64,
    title: String,
}

fn visit_dirs(
    dir: &Path,
    cb: &dyn Fn(&DirEntry, &Regex) -> Result<Vec<Note>>,
) -> Result<Vec<Note>> {
    let re = Regex::new(r"^#+ +([[:alpha:]]{3}) (\d{2}) ([[:alpha:]]{3}) (\d{4}) +(.*)")?;

    let mut notes: Vec<Note> = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                for note in cb(&entry, &re)? {
                    notes.push(note);
                }
            }
        }
    }
    notes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(notes)
}

fn visit_file(entry: &DirEntry, re: &Regex) -> Result<Vec<Note>> {
    let f = File::open(entry.path())?;
    let reader = BufReader::new(&f);

    let mut notes = vec![];
    let mut line_nr = 1;

    for line in reader.lines() {
        let line = line?;
        for cap in re.captures_iter(&line) {
            let month = match &cap[3] {
                "Jan" => 1,
                "Feb" => 2,
                "Mar" => 3,
                "Apr" => 4,
                "May" => 5,
                "Jun" => 6,
                "Jul" => 7,
                "Aug" => 8,
                "Sep" => 9,
                "Oct" => 10,
                "Nov" => 11,
                "Dec" => 12,
                _ => anyhow::bail!("Invalid month: {} in {}", &cap[3], &line),
            };

            let dt = Utc
                .ymd(cap[4].parse::<i32>()?, month, cap[2].parse::<u32>()?)
                .and_hms(12, 12, 12);
            notes.push(Note {
                path: entry.path().to_string_lossy().to_string(),
                line: line_nr,
                timestamp: dt.timestamp(),
                title: cap[5].to_string(),
            })
        }
        line_nr += 1;
    }
    Ok(notes)
}

fn main() -> Result<()> {
    let notes = visit_dirs(Path::new("/home/jari/notes"), &visit_file)?;

    for note in notes {
        println!(
            "{}:{}:{}, {}",
            note.path, note.line, note.timestamp, note.title
        );
    }
    Ok(())
}
