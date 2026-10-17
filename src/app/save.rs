use std::fs;
use std::path::{Path, PathBuf};

const FILE: &str = "doppio.sav";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Save {
    pub unlocked: u8,
    pub tips: u32,
}

pub fn decode(bytes: &[u8], phases: u8) -> Save {
    let unlocked = |n: u8| n.clamp(1, phases.max(1));
    match bytes {
        [n] if *n >= 1 => Save { unlocked: unlocked(*n), tips: 0 },
        [n, a, b, c, d] if *n >= 1 => Save { unlocked: unlocked(*n), tips: u32::from_le_bytes([*a, *b, *c, *d]) },
        _ => Save { unlocked: 1, tips: 0 },
    }
}

pub fn encode(s: Save) -> [u8; 5] {
    let t = s.tips.to_le_bytes();
    [s.unlocked, t[0], t[1], t[2], t[3]]
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

pub fn load(dir: Option<&Path>, phases: u8) -> Save {
    dir.and_then(|d| fs::read(path(d)).ok()).map_or(Save { unlocked: 1, tips: 0 }, |b| decode(&b, phases))
}

pub fn store(dir: Option<&Path>, s: Save) -> std::io::Result<()> {
    let dir = dir.ok_or(std::io::ErrorKind::NotFound)?;
    let tmp = dir.join("doppio.sav.tmp");
    fs::write(&tmp, encode(s)).and_then(|_| fs::rename(&tmp, path(dir)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn up(n: u8) -> Save {
        Save { unlocked: n, tips: 0 }
    }

    #[test]
    fn decodes_and_survives_garbage() {
        assert_eq!(decode(&[3], 5), up(3));
        assert_eq!(decode(&[9], 5), up(5));
        assert_eq!(decode(&[0], 5), up(1));
        assert_eq!(decode(&[], 5), up(1));
        assert_eq!(decode(&[2, 2], 5), up(1));
        assert_eq!(decode(b"garbage", 5), up(1));
        assert_eq!(decode(&[4], 0), up(1));
        assert_eq!(decode(&encode(Save { unlocked: 4, tips: 0x8000_0105 }), 5), Save { unlocked: 4, tips: 0x8000_0105 });
    }

    #[test]
    fn roundtrips_through_the_save_dir() {
        let dir = std::env::temp_dir().join(format!("doppio-save-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(load(Some(&dir), 5), up(1));
        store(Some(&dir), Save { unlocked: 4, tips: 9 }).unwrap();
        assert_eq!(load(Some(&dir), 5), Save { unlocked: 4, tips: 9 });
        assert_eq!(load(Some(&dir), 2), Save { unlocked: 2, tips: 9 });
        fs::write(path(&dir), [3]).unwrap();
        assert_eq!(load(Some(&dir), 5), up(3));
        fs::write(path(&dir), b"\xff\x00junk").unwrap();
        assert_eq!(load(Some(&dir), 5), up(1));
        fs::remove_file(path(&dir)).unwrap();
        fs::create_dir(path(&dir)).unwrap();
        assert_eq!(load(Some(&dir), 5), up(1));
        assert!(store(Some(&dir), up(2)).is_err());
        assert!(store(None, up(2)).is_err());
        assert_eq!(load(None, 5), up(1));
        fs::remove_dir_all(&dir).unwrap();
    }
}
