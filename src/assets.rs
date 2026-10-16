pub static TEXTURES: &[(&str, &[u8])] = &[
    ("azulejo", include_bytes!("../assets/tex/azulejo.tex")),
    ("ceramica", include_bytes!("../assets/tex/ceramica.tex")),
    ("ceramica_barro", include_bytes!("../assets/tex/ceramica_barro.tex")),
    ("colcha", include_bytes!("../assets/tex/colcha.tex")),
    ("concreto", include_bytes!("../assets/tex/concreto.tex")),
    ("croche", include_bytes!("../assets/tex/croche.tex")),
    ("forro", include_bytes!("../assets/tex/forro.tex")),
    ("grama", include_bytes!("../assets/tex/grama.tex")),
    ("lajota", include_bytes!("../assets/tex/lajota.tex")),
    ("madeira_escura", include_bytes!("../assets/tex/madeira_escura.tex")),
    ("madeira_porta", include_bytes!("../assets/tex/madeira_porta.tex")),
    ("metal", include_bytes!("../assets/tex/metal.tex")),
    ("papel_floral", include_bytes!("../assets/tex/papel_floral.tex")),
    ("papel_jornal", include_bytes!("../assets/tex/papel_jornal.tex")),
    ("quadro_santo", include_bytes!("../assets/tex/quadro_santo.tex")),
    ("reboco", include_bytes!("../assets/tex/reboco.tex")),
    ("reboco_mofo", include_bytes!("../assets/tex/reboco_mofo.tex")),
    ("relogio", include_bytes!("../assets/tex/relogio.tex")),
    ("taco", include_bytes!("../assets/tex/taco.tex")),
    ("tecido_sofa", include_bytes!("../assets/tex/tecido_sofa.tex")),
    ("terra", include_bytes!("../assets/tex/terra.tex")),
    ("tijolo", include_bytes!("../assets/tex/tijolo.tex")),
    ("vidro_noite", include_bytes!("../assets/tex/vidro_noite.tex")),
];

#[cfg(test)]
mod tests {
    use super::TEXTURES;

    #[test]
    fn sorted_and_well_formed() {
        assert!(TEXTURES.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(TEXTURES.iter().all(|(_, t)| {
            (5..=7).contains(&t[0]) && (5..=7).contains(&t[1]) && t.len() == 34 + ((1usize << t[0]) << t[1]) / 2
        }));
    }
}
