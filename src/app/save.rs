use std::fs;
use std::path::{Path, PathBuf};

const FILE: &str = "doppio.sav";

pub fn decode(bytes: &[u8], phases: u8) -> u8 {
    match bytes {
        [n] if *n >= 1 => (*n).min(phases.max(1)),
        _ => 1,
    }
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

pub fn load(dir: Option<&Path>, phases: u8) -> u8 {
    dir.and_then(|d| fs::read(path(d)).ok()).map_or(1, |b| decode(&b, phases))
}

pub fn store(dir: Option<&Path>, unlocked: u8) -> std::io::Result<()> {
    let dir = dir.ok_or(std::io::ErrorKind::NotFound)?;
    let tmp = dir.join("doppio.sav.tmp");
    fs::write(&tmp, [unlocked]).and_then(|_| fs::rename(&tmp, path(dir)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_and_survives_garbage() {
        assert_eq!(decode(&[3], 5), 3);
        assert_eq!(decode(&[9], 5), 5);
        assert_eq!(decode(&[0], 5), 1);
        assert_eq!(decode(&[], 5), 1);
        assert_eq!(decode(&[2, 2], 5), 1);
        assert_eq!(decode(b"garbage", 5), 1);
        assert_eq!(decode(&[4], 0), 1);
    }

    #[test]
    fn roundtrips_through_the_save_dir() {
        let dir = std::env::temp_dir().join(format!("doppio-save-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(load(Some(&dir), 5), 1);
        store(Some(&dir), 4).unwrap();
        assert_eq!(load(Some(&dir), 5), 4);
        assert_eq!(load(Some(&dir), 2), 2);
        fs::write(path(&dir), b"\xff\x00junk").unwrap();
        assert_eq!(load(Some(&dir), 5), 1);
        fs::remove_file(path(&dir)).unwrap();
        fs::create_dir(path(&dir)).unwrap();
        assert_eq!(load(Some(&dir), 5), 1);
        assert!(store(Some(&dir), 2).is_err());
        assert!(store(None, 2).is_err());
        assert_eq!(load(None, 5), 1);
        fs::remove_dir_all(&dir).unwrap();
    }
}
