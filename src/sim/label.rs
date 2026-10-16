const TITLES: &[(&str, &str)] = &[
    ("area_servico", "área de serviço"),
    ("banheiro", "banheiro"),
    ("corredor", "corredor"),
    ("corredor_lateral", "corredor lateral"),
    ("cozinha", "cozinha"),
    ("despensa", "despensa"),
    ("quarto", "quarto"),
    ("quarto_vo", "quarto da vó"),
    ("quintal", "quintal"),
    ("sala_estar", "sala de estar"),
    ("sala_jantar", "sala de jantar"),
    ("varanda", "varanda"),
];

pub fn room_title(name: &'static str) -> &'static str {
    TITLES.binary_search_by_key(&name, |&(k, _)| k).map_or(name, |i| TITLES[i].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sobrado_room_has_a_title() {
        assert!(TITLES.windows(2).all(|w| w[0].0 < w[1].0));
        crate::map::sobrado().rooms.iter().for_each(|r| assert!(TITLES.iter().any(|(k, _)| *k == r.name), "{}", r.name));
        assert_eq!(room_title("quarto_vo"), "quarto da vó");
        assert_eq!(room_title("porao"), "porao");
    }
}
