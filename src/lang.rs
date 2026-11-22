//! The European releases this makes mods of, and what each mod has that
//! is not on the discs: the speakers' names (pictures there, read by eye)
//! and the loading captions' places the release leaves in English
//! (translated for this program).

pub struct Release {
    /// Its executable's name on disc 1.
    pub exe: &'static str,
    /// The mod's folder in PE2PC's mods/.
    pub folder: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub speakers: &'static str,
    pub places: &'static str,
}

pub const RELEASES: [Release; 4] = [
    Release {
        exe: "SLES_025.59",
        folder: "francais",
        name: "Français (version PAL)",
        description: "Dialogues, noms, menus, objets et images de la version française (SLES-02559), importés des disques PAL.",
        speakers: include_str!("../res/fr/speakers.toml"),
        places: include_str!("../res/fr/places.toml"),
    },
    Release {
        exe: "SLES_025.60",
        folder: "deutsch",
        name: "Deutsch (PAL-Version)",
        description: "Dialoge, Namen, Menüs, Gegenstände und Bilder der deutschen Version (SLES-02560), von den PAL-Discs importiert.",
        speakers: include_str!("../res/de/speakers.toml"),
        places: include_str!("../res/de/places.toml"),
    },
    Release {
        exe: "SLES_025.61",
        folder: "espanol",
        name: "Español (versión PAL)",
        description: "Diálogos, nombres, menús, objetos e imágenes de la versión española (SLES-02561), importados de los discos PAL.",
        speakers: include_str!("../res/es/speakers.toml"),
        places: include_str!("../res/es/places.toml"),
    },
    Release {
        exe: "SLES_025.62",
        folder: "italiano",
        name: "Italiano (versione PAL)",
        description: "Dialoghi, nomi, menu, oggetti e immagini della versione italiana (SLES-02562), importati dai dischi PAL.",
        speakers: include_str!("../res/it/speakers.toml"),
        places: include_str!("../res/it/places.toml"),
    },
];

/// The release whose disc 1 has executable `exe`.
pub fn release(exe: &str) -> Option<&'static Release> {
    RELEASES.iter().find(|r| r.exe.eq_ignore_ascii_case(exe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_reads() {
        for r in &RELEASES {
            let speakers: toml::Table = r.speakers.parse().expect("speakers");
            let places: toml::Table = r.places.parse().expect("places");
            assert!(!speakers.is_empty() && !places.is_empty(), "{}", r.folder);
        }
    }
}
