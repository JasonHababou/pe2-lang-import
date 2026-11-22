//! A language mod from two extracts: the game's (USA) and a European
//! release's.

use std::path::Path;

use crate::Result;
use crate::dumps::Version;
use crate::lang::Release;
use crate::{dialogue, release, strings};

/// What the import made.
pub struct Made {
    pub release: &'static Release,
    pub lines: usize,
    pub texts: usize,
    pub details: usize,
    pub captions: usize,
    pub pictures: usize,
}

/// The release `dumps` is of.
pub fn release_of(dumps: &Path) -> Result<&'static Release> {
    let version = Version::open(dumps)?;
    crate::lang::release(&version.exe_name)
        .ok_or_else(|| format!("{}: not a European release this knows (French, German, Spanish or Italian)", version.exe_name).into())
}

/// Makes the mod in `out` (an empty folder, or one it made before) from the
/// game's extract `game` and the release's `other`; `name` is its name.
pub fn make(game: &Path, other: &Path, out: &Path, name: Option<&str>) -> Result<Made> {
    let (mine, theirs) = (Version::open(game)?, Version::open(other)?);
    let release = release_of(other)?;
    let name = name.unwrap_or(release.name);
    for part in ["text", "strings", "files"] {
        let dir = out.join(part);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
    }
    std::fs::create_dir_all(out)?;
    let mut report = Vec::new();
    let (_, lines) = dialogue::import(&game.join("text"), &other.join("text"), out, name, &mut report)?;
    std::fs::write(out.join("text").join("speakers.toml"), release.speakers)?;
    let (found, pictures) = strings::import(&mine, &theirs, out, &mut report)?;
    let places: toml::Table = release.places.parse()?;
    let (texts, details, captions) = release::import(&mine, &theirs, &found, &places, out, &mut report)?;
    std::fs::write(
        out.join("mod.toml"),
        format!("name = {}\ndescription = {}\nenabled = true\n", toml_string(name), toml_string(release.description)),
    )?;
    std::fs::write(out.join("report.txt"), report.join("\n") + "\n")?;
    Ok(Made { release, lines, texts, details, captions, pictures })
}

fn toml_string(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}
