include!(concat!(env!("OUT_DIR"), "/textures.rs"));

#[cfg(test)]
mod tests {
    use super::TEXTURES;

    #[test]
    fn sorted_and_well_formed() {
        assert!(TEXTURES.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(TEXTURES.iter().all(|(_, t)| {
            (5..=8).contains(&t[0]) && (5..=8).contains(&t[1]) && t.len() == 34 + ((1usize << t[0]) << t[1]) / 2
        }));
    }
}
