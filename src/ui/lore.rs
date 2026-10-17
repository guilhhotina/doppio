use crate::map::{ItemKind, LegendKind};

pub struct Character {
    pub name: &'static str,
    pub short: &'static str,
    pub lore: &'static str,
    pub perk: &'static str,
}

pub struct Legend {
    pub name: &'static str,
    pub line: &'static str,
    pub lore: &'static str,
}

pub const CHARACTERS: [Character; 4] = [
    Character {
        name: "Dona Benedita",
        short: "Benedita",
        lore: "Benzedeira, 71 anos. Conhece toda lenda.",
        perk: "Acorda em 3 s e sente o lado da lenda.",
    },
    Character {
        name: "Tonho",
        short: "Tonho",
        lore: "Coroinha, 12 anos, neto da Dona Benedita.",
        perk: "Anda sem barulho e se esconde num instante.",
    },
    Character {
        name: "Jairo",
        short: "Jairo",
        lore: "Caminhoneiro, 45. Ficou preso na cidade.",
        perk: "Carrega dois itens e força portas emperradas.",
    },
    Character {
        name: "Bia",
        short: "Bia",
        lore: "Estudante de jornalismo, 22. Filma lendas.",
        perk: "Lanterna dura o triplo; vê itens de longe.",
    },
];

pub fn character(id: u8) -> &'static Character {
    &CHARACTERS[id as usize % CHARACTERS.len()]
}

pub fn legend(kind: LegendKind) -> Legend {
    match kind {
        LegendKind::CorpoSeco => Legend {
            name: "Corpo-seco",
            line: "Nem a terra quis o homem ruim. Ele odeia sal.",
            lore: "Dizem que foi um homem tão ruim que a terra cuspiu o corpo de volta. Anda seco e devagar pelas casas. Sal na porta e cruz na mão o seguram.",
        },
        LegendKind::Cabra => Legend {
            name: "Cabra Cabriola",
            line: "Bale na madrugada e solta fogo pela boca.",
            lore: "Bicho de chifre torto que bale pela madrugada atrás de quem está na rua. Quando corre, cospe fogo e ilumina tudo. Um balde d'água apaga o fogo dela.",
        },
        LegendKind::Mula => Legend {
            name: "Mula sem cabeça",
            line: "Galopa reto, soltando fogo. Não sabe virar.",
            lore: "Onde devia estar a cabeça, só há fogo. Galopa pela igreja nas noites santas. Corre em linha reta e não sabe virar: saia do caminho. Tire o freio dela.",
        },
        LegendKind::Matinta => Legend {
            name: "Matinta Perera",
            line: "Assobia na mata. Fumo prometido a acalma.",
            lore: "Velha que vira pássaro e assobia fino na beira do rio, a noite inteira. Quem ouve o assobio promete fumo. Deixe o fumo de rolo onde ela dorme.",
        },
    }
}

pub fn item(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Chave => "chave",
        ItemKind::Alicate => "alicate",
        ItemKind::Martelo => "martelo",
        ItemKind::PeDeCabra => "pé de cabra",
        ItemKind::ChaveDeFenda => "chave de fenda",
        ItemKind::Fusivel => "fusível",
        ItemKind::Sal => "sal grosso",
        ItemKind::Vela => "vela",
        ItemKind::Fosforo => "fósforos",
        ItemKind::Tabaco => "fumo de rolo",
        ItemKind::Balde => "balde d'água",
        ItemKind::Freio => "freio de ferro",
        ItemKind::Ossos => "ossos",
        ItemKind::Cruz => "cruz",
        ItemKind::BilheteOnibus => "passagem",
        ItemKind::Manivela => "manivela",
        ItemKind::Remo => "remo",
        ItemKind::LanternaPilha => "pilhas",
        ItemKind::Foto => "foto antiga",
    }
}

pub const ITEMS: [ItemKind; 19] = [
    ItemKind::Chave,
    ItemKind::Alicate,
    ItemKind::Martelo,
    ItemKind::PeDeCabra,
    ItemKind::ChaveDeFenda,
    ItemKind::Fusivel,
    ItemKind::Sal,
    ItemKind::Vela,
    ItemKind::Fosforo,
    ItemKind::Tabaco,
    ItemKind::Balde,
    ItemKind::Freio,
    ItemKind::Ossos,
    ItemKind::Cruz,
    ItemKind::BilheteOnibus,
    ItemKind::Manivela,
    ItemKind::Remo,
    ItemKind::LanternaPilha,
    ItemKind::Foto,
];

pub const LEGENDS: [LegendKind; 4] = [LegendKind::CorpoSeco, LegendKind::Cabra, LegendKind::Mula, LegendKind::Matinta];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_fit_the_screen() {
        CHARACTERS.iter().for_each(|c| {
            assert!(c.short.chars().count() <= 10 && c.name.chars().count() <= 16);
            assert!(c.lore.chars().count() <= 45 && c.perk.chars().count() <= 45);
        });
        LEGENDS.iter().map(|&k| legend(k)).for_each(|l| {
            assert!(l.line.chars().count() <= 46, "{}", l.line);
            assert!(crate::ui::wrap(l.lore, 202, crate::ui::Body).count() <= 9);
        });
        assert!(ITEMS.iter().all(|&k| item(k).chars().count() <= 16));
        assert_eq!(character(6).short, "Jairo");
    }
}
