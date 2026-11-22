//! The whole import: both releases extracted by PE2PC (`pe2 --extract`,
//! each into a work folder of its own), the mod made from them into PE2PC's
//! mods folder, and the movies with text converted by ffmpeg.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Result;
use crate::import::{self, Made};

/// The movies whose pictures have text (the title menu's, the prologue's),
/// by the hash PE2PC names them by (the same in all releases: of their
/// first frame).
const MOVIES_WITH_TEXT: [&str; 2] = ["0d17397b6feab122", "39d9715252254830"];

pub struct Job {
    /// PE2PC's program.
    pub pe2: PathBuf,
    /// PE2PC's data folder: the mod goes to its mods/.
    pub data: PathBuf,
    pub usa: Vec<PathBuf>,
    pub release: Vec<PathBuf>,
    /// ffmpeg's program, if there is one.
    pub ffmpeg: Option<PathBuf>,
    /// Where the extracts go (removed after).
    pub work: PathBuf,
}

pub enum Step {
    ExtractUsa,
    ExtractRelease,
    Import,
    Movies,
}

pub enum Event {
    Step(Step),
    /// A line of PE2PC's extract.
    Line(String),
    Done(Result<Done>),
}

pub struct Done {
    pub made: Made,
    pub folder: PathBuf,
    /// The movies converted, or why not.
    pub movies: std::result::Result<usize, String>,
}

/// Runs `job`, telling `tell` how it goes.
pub fn run(job: &Job, tell: &dyn Fn(Event)) {
    let result = run_steps(job, tell);
    let _ = std::fs::remove_dir_all(&job.work);
    tell(Event::Done(result));
}

fn run_steps(job: &Job, tell: &dyn Fn(Event)) -> Result<Done> {
    let (usa, other) = (job.work.join("usa"), job.work.join("release"));
    tell(Event::Step(Step::ExtractUsa));
    extract(job, &usa, &job.usa, "files,textures,texts", tell)?;
    tell(Event::Step(Step::ExtractRelease));
    extract(job, &other, &job.release, "files,textures,texts,movies", tell)?;
    tell(Event::Step(Step::Import));
    let (usa, other) = (usa.join("dumps"), other.join("dumps"));
    let release = import::release_of(&other)?;
    let folder = job.data.join("mods").join(release.folder);
    let made = import::make(&usa, &other, &folder, None)?;
    tell(Event::Step(Step::Movies));
    let movies = match &job.ffmpeg {
        Some(ffmpeg) => movies(ffmpeg, &other.join("movies"), &folder.join("movies")),
        None => Err("ffmpeg not found".into()),
    };
    Ok(Done { made, folder, movies })
}

/// `pe2 --extract <parts> <discs>` into `home`'s dumps/.
fn extract(job: &Job, home: &Path, discs: &[PathBuf], parts: &str, tell: &dyn Fn(Event)) -> Result<()> {
    std::fs::create_dir_all(home)?;
    let mut command = Command::new(&job.pe2);
    command.env("PE2_HOME", home).arg("--extract").arg(parts).args(discs);
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    no_window(&mut command);
    let mut child = command.spawn().map_err(|e| format!("{}: {e}", job.pe2.display()))?;
    let mut last = Vec::new();
    if let Some(err) = child.stderr.take() {
        for line in std::io::BufReader::new(err).lines().map_while(std::io::Result::ok) {
            // without the log's time stamp
            let line = line.split_once("] ").filter(|_| line.starts_with('[')).map_or(line.as_str(), |l| l.1).to_string();
            tell(Event::Line(line.clone()));
            last.push(line);
            if last.len() > 5 {
                last.remove(0);
            }
        }
    }
    if !child.wait()?.success() {
        return Err(format!("PE2PC's extract failed:\n{}", last.join("\n")).into());
    }
    Ok(())
}

/// The movies with text, from the release's extract (`<hash>.str`) to the
/// mod's `<hash>.mp4`. Answers how many, or why none.
fn movies(ffmpeg: &Path, from: &Path, to: &Path) -> std::result::Result<usize, String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut made = 0;
    for hash in MOVIES_WITH_TEXT {
        let source = from.join(format!("{hash}.str"));
        if !source.exists() {
            continue;
        }
        let mut command = Command::new(ffmpeg);
        command.args(["-loglevel", "error", "-y", "-i"]).arg(&source);
        command.args(["-c:v", "libx264", "-bf", "0", "-pix_fmt", "yuv420p", "-crf", "16", "-c:a", "aac", "-b:a", "192k"]);
        command.arg(to.join(format!("{hash}.mp4"))).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
        no_window(&mut command);
        let out = command.output().map_err(|e| format!("{}: {e}", ffmpeg.display()))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        made += 1;
    }
    Ok(made)
}

/// No console window for a program started from the window.
fn no_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}

/// ffmpeg on the PATH, if it is there.
pub fn find_ffmpeg() -> Option<PathBuf> {
    let name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(name)).find(|p| p.is_file())
}
