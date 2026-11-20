//! The channels, compiled in: who they are and where their live comes from.

use crate::model::{Catalog, Channel};

/// How a channel's live address is found at play time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A Dailymotion live, with the page it must be embedded in when its owner restricts embedding.
    Dailymotion {
        video: &'static str,
        embedder: Option<&'static str>,
    },
    /// A France Télévisions live, by broadcast id.
    FranceTv {
        si_id: &'static str,
    },
    /// A bfmtv.com live page, by path.
    Bfm {
        path: &'static str,
    },
    Kick {
        slug: &'static str,
    },
    Equidia {
        name: &'static str,
    },
    EasyBroadcast {
        tenant: &'static str,
        event: &'static str,
    },
    /// A page with an `HDR_VISIO` player.
    HdrVisio {
        page: &'static str,
    },
    /// An Ultimedia (Digiteka) player page.
    Digiteka {
        iframe: &'static str,
    },
    TvMonaco,
    /// An address that does not change.
    Static {
        url: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub number: Option<i64>,
    pub logo: Option<&'static str>,
    pub source: Source,
}

macro_rules! telerama {
    ($n:literal) => {
        Some(concat!("https://focus.telerama.fr/500x500/0000/00/01/clear-", $n, ".png"))
    };
}

macro_rules! ftv_logo {
    ($path:literal) => {
        Some(concat!("https://medias.france.tv/", $path))
    };
}

const FRANCE_3_LOGO: Option<&str> = ftv_logo!("2cSonvrdpEyEdrJ-mpSlq6VRyQo/400x400/filters:quality(85)/i/v/v/phpjxpvvi.png");

const fn ch(id: &'static str, name: &'static str, group: &'static str, logo: Option<&'static str>, source: Source) -> Entry {
    Entry { id, name, group, number: None, logo, source }
}

const fn tnt(number: i64, entry: Entry) -> Entry {
    Entry { number: Some(number), ..entry }
}

const fn dm(video: &'static str) -> Source {
    Source::Dailymotion { video, embedder: None }
}

const fn dm_in(video: &'static str, embedder: &'static str) -> Source {
    Source::Dailymotion { video, embedder: Some(embedder) }
}

const fn ftv(si_id: &'static str) -> Source {
    Source::FranceTv { si_id }
}

const fn bfm(path: &'static str) -> Source {
    Source::Bfm { path }
}

const fn equidia(name: &'static str) -> Source {
    Source::Equidia { name }
}

const fn fixed(url: &'static str) -> Source {
    Source::Static { url }
}

const fn france_3(id: &'static str, name: &'static str, si_id: &'static str) -> Entry {
    ch(id, name, "Regional", FRANCE_3_LOGO, ftv(si_id))
}

const fn bfm_local(id: &'static str, name: &'static str, path: &'static str) -> Entry {
    ch(id, name, "Regional", None, bfm(path))
}

pub const CHANNELS: &[Entry] = &[
    // National
    tnt(
        2,
        ch(
            "france-2",
            "France 2",
            "National",
            ftv_logo!("v25lgE50IPcm5SkmaN7WLDiOuEo/400x400/filters:quality(85)/c/7/t/php6kjt7c.png"),
            ftv("006194ea-117d-4bcf-94a9-153d999c59ae"),
        ),
    ),
    tnt(3, ch("france-3", "France 3", "National", FRANCE_3_LOGO, ftv("29bdf749-7082-4426-a4f3-595cc436aa0d"))),
    tnt(
        4,
        ch(
            "france-4",
            "France 4",
            "National",
            ftv_logo!("gPIpmplXDRY_1BHEJOKsrR7ZpPU/400x400/filters:quality(85)/g/7/f/php2isf7g.png"),
            ftv("9a6a7670-dde9-4264-adbc-55b89558594b"),
        ),
    ),
    tnt(
        5,
        ch(
            "france-5",
            "France 5",
            "National",
            ftv_logo!("hztfOht-0xZ-zZvXJoQtjd9s1IM/400x400/filters:quality(85)/3/v/x/phptcexv3.png"),
            ftv("45007886-f3ff-4b3e-9706-1ef1014c5a60"),
        ),
    ),
    tnt(
        7,
        ch(
            "arte",
            "Arte",
            "National",
            ftv_logo!("v7wcepEnovdAvtlszxI7Pku2j8w/400x400/filters:quality(85)/6/l/v/phps6hvl6.jpg"),
            ftv("7e3d129e-9c17-4d49-a25e-5e913ba91e35"),
        ),
    ),
    tnt(
        8,
        ch(
            "lcp",
            "LCP / Public Sénat",
            "National",
            ftv_logo!("UrAjn5SvjixHhjjqutdNW3yZbTs/400x400/filters:quality(85)/0/w/w/phptavww0.jpg"),
            ftv("733d60e7-914f-40ed-b296-0ddbc4b6d414"),
        ),
    ),
    ch(
        "public-senat",
        "Public Sénat",
        "National",
        Some("https://s2.dmcdn.net/u/11mWs1h00UU0dxItA/720x720"),
        dm_in("xkxbzc", "https://www.publicsenat.fr/"),
    ),
    tnt(18, ch("t18", "T18", "National", Some("https://s1.dmcdn.net/u/CRoWo1g_t-yODpTe1/720x720"), dm_in("x9jhhyc", "https://t18.fr/"))),
    // News
    tnt(13, ch("bfmtv", "BFM TV", "News", telerama!(481), bfm("en-direct/"))),
    tnt(14, ch("cnews", "CNews", "News", Some("https://s1.dmcdn.net/u/DhmL1h00UJEql98f/720x720"), dm("x3b68jn"))),
    tnt(
        16,
        ch(
            "franceinfo",
            "franceinfo:",
            "News",
            ftv_logo!("E4mTVkgkoJVsZkf9Tj9Aw-Q8uNQ/400x400/filters:quality(85)/5/u/b/phpp1cbu5.jpg"),
            ftv("35be22fb-1569-43ff-857c-99bf81defa2e"),
        ),
    ),
    ch("bfm-business", "BFM Business", "News", telerama!(1073), bfm("economie/en-direct/")),
    ch("bfm2", "BFM2", "News", None, bfm("en-direct/bfm2/")),
    ch("bsmart", "B Smart", "News", Some("https://s1.dmcdn.net/u/8W2l91h00DehlXIp2/720x720"), dm("x7tn4fp")),
    ch(
        "france-24",
        "France 24",
        "News",
        ftv_logo!("7ua0ajgM4x6MMwCKKGlzyaurqtU/400x400/filters:quality(85)/g/2/j/php8qkj2g.jpg"),
        ftv("da9e13f0-42f3-4618-9954-9f5b63e0fe1f"),
    ),
    // Sport
    tnt(
        21,
        ch(
            "lequipe",
            "L'Équipe",
            "Sport",
            Some("https://s2.dmcdn.net/u/5LK8J1g_ynQpVVSwY/720x720"),
            dm_in("x2lefik", "https://www.lequipe.fr/tv/videos/live/x2lefik"),
        ),
    ),
    ch("francetv-sport", "France TV Sport", "Sport", None, ftv("33a20612-120a-4e60-bd96-225146e4cf0c")),
    ch("sport-en-france", "Sport en France", "Sport", Some("https://s1.dmcdn.net/u/8Dz9c1g_yycqzo5QV/720x720"), dm("x8sayn8")),
    ch("equidia", "Equidia", "Sport", telerama!(64), equidia("equidia-live2")),
    ch("equidia-racing-trot", "Equidia Racing Trot", "Sport", None, equidia("equidia-racingtrot")),
    ch("equidia-racing-mag", "Equidia Racing Mag", "Sport", None, equidia("equidia-racingmag")),
    ch("equidia-racing-galop", "Equidia Racing Galop", "Sport", None, equidia("equidia-racinggalop")),
    ch("equidia-racing-1", "Equidia Racing 1", "Sport", None, equidia("equidia-racing1")),
    ch("equidia-racing-2", "Equidia Racing 2", "Sport", None, equidia("equidia-racing2")),
    ch("equidia-racing-3", "Equidia Racing 3", "Sport", None, equidia("equidia-racing3")),
    ch("equidia-racing-4", "Equidia Racing 4", "Sport", None, equidia("equidia-racing4")),
    ch("equidia-racing-5", "Equidia Racing 5", "Sport", None, equidia("equidia-racing5")),
    ch("equidia-racing-6", "Equidia Racing 6", "Sport", None, equidia("equidia-racing6")),
    ch("equidia-racing-7", "Equidia Racing 7", "Sport", None, equidia("equidia-racing7")),
    ch("equidia-racing-8", "Equidia Racing 8", "Sport", None, equidia("equidia-racing8")),
    // Thematic
    ch("francetv-docs", "France TV Docs", "Thematic", None, ftv("1e4bd223-8b32-42a0-bc24-12edb323eec7")),
    ch("francetv-series", "France TV Séries", "Thematic", None, ftv("61c8c8fd-2454-4fee-80cb-38419e78eea2")),
    ch("la-chaine-des-maternelles", "La chaîne des Maternelles", "Thematic", None, ftv("b1f511a3-232a-48fb-acd0-09407646de2f")),
    ch(
        "ina",
        "INA",
        "Thematic",
        ftv_logo!("1u0HqgdutwU_L3xnJqwUBuwkNHg/400x400/filters:quality(85)/r/r/j/phpltsjrr.jpg"),
        ftv("ce51459d-2ada-484d-98e2-c8b24e95912d"),
    ),
    ch(
        "mieux",
        "Mieux",
        "Thematic",
        ftv_logo!("ByR0XZQ07tJcPl3atMd0Qn4jwGw/400x400/filters:quality(85)/d/s/2/phpesu2sd.jpg"),
        ftv("c7471f59-f1c4-4a73-9184-8ef2ce96d6d8"),
    ),
    ch("kto", "KTO", "Thematic", telerama!(110), fixed("https://livekto.akamaized.net/hls/live/20000018/ktotv/master.m3u8")),
    ch("gong", "Gong", "Thematic", None, fixed("https://amg01596-gongnetworks-gong-ono-vh5f2.amagi.tv/playlist.m3u8")),
    // International
    ch(
        "tv5monde",
        "TV5Monde",
        "International",
        ftv_logo!("gky_T4ERU0ImeLxI-7piK08RJ14/400x400/filters:quality(85)/n/9/o/phpplvo9n.jpg"),
        ftv("c2b61cd9-4923-44e9-820b-b7365734d507"),
    ),
    // Radio
    ch("france-inter", "France Inter", "Radio", Some("https://s2.dmcdn.net/u/HT2x1g_wdL-cswGC/720x720"), dm("x17qw0a")),
    ch(
        "europe-1",
        "Europe 1",
        "Radio",
        Some("https://s2.dmcdn.net/u/1W-uR1h00NgAsWKfA/720x720"),
        dm_in("xqjkfz", "https://www.europe1.fr/"),
    ),
    ch("rtl2", "RTL2", "Radio", Some("https://s2.dmcdn.net/u/2MjtK1h00VuZGpHAN/720x720"), dm("x2tzzpj")),
    ch("fun-radio", "Fun Radio", "Radio", Some("https://s1.dmcdn.net/u/2MYk91h00Vu1EkHjC/720x720"), dm("xxtuy6")),
    ch("sud-radio", "Sud Radio", "Radio", Some("https://s2.dmcdn.net/u/1ERZ11h002-l46o3W/720x720"), dm("x8jqxru")),
    // Regional: France 3 editions
    france_3("france-3-alpes", "France 3 Alpes", "1746574b-ad9b-4484-883d-847cc0051460"),
    france_3("france-3-auvergne", "France 3 Auvergne", "0d44f768-753a-49c3-8ff1-7d9fdeca4dc6"),
    france_3("france-3-rhone-alpes", "France 3 Rhône-Alpes", "08f8633e-4963-4ddc-a2eb-8d5531f5c83f"),
    france_3("france-3-bourgogne", "France 3 Bourgogne", "ce576715-5360-43a9-a11a-6a0f76b09050"),
    france_3("france-3-franche-comte", "France 3 Franche-Comté", "b00cb1d6-3792-4fc1-8ccb-0d65cc2167a0"),
    france_3("france-3-bretagne", "France 3 Bretagne", "feb87a79-eccb-4091-bfe4-bdb1d07c585b"),
    france_3("france-3-centre-val-de-loire", "France 3 Centre-Val de Loire", "8abdcdea-e644-4c17-841a-8d838dc9ec02"),
    france_3("france-3-corse-via-stella", "France 3 Corse ViaStella", "84591f36-2cc3-4ebb-9aba-4192295c4055"),
    france_3("france-3-alsace", "France 3 Alsace", "5eb38bef-8cdb-40aa-b5ca-02d2c79b3055"),
    france_3("france-3-champagne-ardenne", "France 3 Champagne-Ardenne", "fdb92cee-b9df-4ea3-9f6f-0a683b4fb9a5"),
    france_3("france-3-lorraine", "France 3 Lorraine", "7e5cbdf4-1c89-48bd-bbf1-d8144bdb5d02"),
    france_3("france-3-nord-pas-de-calais", "France 3 Nord Pas-de-Calais", "3c7d6c36-68c8-45d8-b919-0911c44b81c6"),
    france_3("france-3-picardie", "France 3 Picardie", "94dd505f-23b5-48c9-ac20-249fc0601b53"),
    france_3("france-3-basse-normandie", "France 3 Basse-Normandie", "6874817f-ecf4-453b-9c4c-8c72f59352de"),
    france_3("france-3-haute-normandie", "France 3 Haute-Normandie", "891e0b91-0b9c-4398-ba26-baa1ef80c55c"),
    france_3("france-3-aquitaine", "France 3 Aquitaine", "892351af-cd6e-4ca9-a40b-250408d2ba8f"),
    france_3("france-3-limousin", "France 3 Limousin", "45e0880a-0d47-4e16-ada6-d6e1e9326a79"),
    france_3("france-3-noa", "France 3 NoA", "ea65a32f-11ed-11ea-8b6e-0050568b2441"),
    france_3("france-3-poitou-charentes", "France 3 Poitou-Charentes", "03fa6a93-c4b7-4616-944b-faa6914bea6e"),
    france_3("france-3-languedoc-roussillon", "France 3 Languedoc-Roussillon", "3c8bf404-d859-4312-b3a8-7e63edbba2e9"),
    france_3("france-3-midi-pyrenees", "France 3 Midi-Pyrénées", "807d966c-1031-42f4-b0b8-14f4c4d840d8"),
    france_3("france-3-paris-ile-de-france", "France 3 Paris Île-de-France", "df2fbe9f-a666-4af2-bfd6-3213189c2078"),
    france_3("france-3-pays-de-la-loire", "France 3 Pays de la Loire", "af5476dd-0164-43d6-8dc8-4f1841bd4cde"),
    france_3("france-3-cote-d-azur", "France 3 Côte d'Azur", "a540113e-a0e0-45fd-a7a8-5d0ccee6f2ba"),
    france_3("france-3-provence-alpes", "France 3 Provence-Alpes", "4931dbef-64e5-4293-8271-56188e794bcf"),
    // Regional: BFM local channels
    bfm_local("bfm-alsace", "BFM Alsace", "alsace/en-direct/"),
    bfm_local("bfm-dici-haute-provence", "BFM DICI Haute-Provence", "bfm-dici/haute-provence/en-direct/"),
    bfm_local("bfm-dici-alpes-du-sud", "BFM DICI Alpes du Sud", "bfm-dici/alpes-du-sud/en-direct/"),
    bfm_local("bfm-grand-lille", "BFM Grand Lille", "grand-lille/en-direct/"),
    bfm_local("bfm-grand-littoral", "BFM Grand Littoral", "grand-littoral/en-direct/"),
    bfm_local("bfm-lyon", "BFM Lyon", "lyon/en-direct/"),
    bfm_local("bfm-marseille-provence", "BFM Marseille Provence", "marseille/en-direct/"),
    bfm_local("bfm-nice-cote-d-azur", "BFM Nice Côte d'Azur", "cote-d-azur/en-direct/"),
    bfm_local("bfm-normandie", "BFM Normandie", "normandie/en-direct/"),
    bfm_local("bfm-toulon-var", "BFM Toulon Var", "var/en-direct/"),
    // Overseas
    ch("reunion-la-1ere", "Réunion La 1ère", "Overseas", telerama!(245), ftv("5e959b18-89d2-458b-8db7-1f1fba23b98f")),
    ch("nouvelle-caledonie-la-1ere", "Nouvelle-Calédonie La 1ère", "Overseas", telerama!(240), ftv("d718bf92-092d-486f-9497-cd8aadeabcee")),
    ch("wallis-et-futuna-la-1ere", "Wallis et Futuna La 1ère", "Overseas", None, ftv("229852d7-0bdc-46c4-aba4-3764d31beec7")),
    ch(
        "antenne-reunion",
        "Antenne Réunion",
        "Overseas",
        None,
        fixed("https://live-antenne-reunion.zeop.tv/live/c3eds/antreunihd/hls_fta/antreunihd.m3u8"),
    ),
    // Local
    ch(
        "matele",
        "viàMaTélé",
        "Local",
        None,
        fixed("https://5dd226f8f01e8.streamlock.net:443/via-matele-live/matelelive_1080/playlist.m3u8"),
    ),
    ch("viavosges", "viàVosges", "Local", None, Source::HdrVisio { page: "https://www.viavosges.tv/Direct.html" }),
    ch("tv7-colmar", "TV7 Colmar", "Local", None, Source::HdrVisio { page: "https://www.tv7.fr/" }),
    ch("8-mont-blanc", "8 Mont-Blanc", "Local", Some("https://s2.dmcdn.net/u/8R0v1g_u3z1P2up0/720x720"), dm("x3wqv8b")),
    ch(
        "moselle-tv",
        "Moselle TV",
        "Local",
        Some("https://s2.dmcdn.net/u/2QNmg1g_uzyxb9AFE/720x720"),
        dm_in("x9lgbik", "https://moselle.tv"),
    ),
    ch("telenantes", "TéléNantes", "Local", Some("https://s1.dmcdn.net/u/H9UA1gyps20lwK_B/720x720"), dm("x5so98l")),
    ch("bip-tv", "BIP TV", "Local", None, fixed("https://hls-biptv.archistream.live/hls/master.m3u8")),
    ch("figaro-tv-idf", "Figaro TV Île-de-France", "Local", None, fixed("https://static.lefigaro.fr/secom/tnt.m3u8")),
    ch(
        "20-minutes-tv-idf",
        "20 Minutes TV Île-de-France",
        "Local",
        None,
        Source::Digiteka { iframe: "https://www.ultimedia.com/deliver/generic/iframe/mdtk/01521514/zone/1/showtitle/1/src/qvrlxxf" },
    ),
    ch("tebeo", "Tébéo", "Local", None, fixed("https://tebeo.streamakaci.tv/tebeo/tebeo-multi/playlist.m3u8")),
    ch("telepaese", "TéléPaese", "Local", None, fixed("https://srv.webtvmanager.fr:3970/live/viatelepaeselive.m3u8")),
    ch(
        "tg-plus",
        "TG+",
        "Local",
        Some("https://files.kick.com/images/user/53234261/profile_image/conversion/ff96cc7f-6301-47a3-93cf-2aee5e23eba8-fullsize.webp"),
        Source::Kick { slug: "tgplus" },
    ),
    ch("tvr", "TVR", "Local", None, Source::EasyBroadcast { tenant: "captaprod", event: "54_tvr_vtucuxt" }),
    ch("tv-monaco", "TV Monaco", "Local", None, Source::TvMonaco),
];

/// What the catalog says a channel streams from: a stand-in, since the live is resolved at play time by channel id.
pub fn link(id: &str) -> String {
    format!("https://antenne.invalid/{id}")
}

pub fn find(id: &str) -> Option<&'static Entry> {
    CHANNELS.iter().find(|e| e.id == id)
}

pub fn load() -> Catalog {
    let channels = CHANNELS
        .iter()
        .map(|e| Channel {
            id: e.id.into(),
            name: e.name.into(),
            number: e.number,
            group: Some(e.group.into()),
            logo: e.logo.map(Into::into),
            stream: link(e.id),
        })
        .collect();
    Catalog { channels }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_plain() {
        let mut seen = HashSet::new();
        for e in CHANNELS {
            assert!(seen.insert(e.id), "{} twice", e.id);
            assert!(e.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{}", e.id);
        }
    }

    #[test]
    fn numbers_are_the_national_tnt_ones() {
        let numbered: Vec<_> = CHANNELS.iter().filter_map(|e| e.number.map(|n| (n, e.id))).collect();
        assert_eq!(
            numbered,
            [
                (2, "france-2"),
                (3, "france-3"),
                (4, "france-4"),
                (5, "france-5"),
                (7, "arte"),
                (8, "lcp"),
                (18, "t18"),
                (13, "bfmtv"),
                (14, "cnews"),
                (16, "franceinfo"),
                (21, "lequipe")
            ]
        );
    }

    #[test]
    fn logos_are_https() {
        assert!(CHANNELS.iter().filter_map(|e| e.logo).all(|l| l.starts_with("https://")));
        assert_eq!(telerama!(4), Some("https://focus.telerama.fr/500x500/0000/00/01/clear-4.png"));
    }

    #[test]
    fn the_catalog_lists_every_channel_by_group() {
        let catalog = load();
        assert_eq!(catalog.channels.len(), CHANNELS.len());
        let cnews = catalog.channels.iter().find(|c| c.id == "cnews").unwrap();
        assert_eq!(cnews.number, Some(14));
        assert_eq!(cnews.group.as_deref(), Some("News"));
        assert_eq!(cnews.stream, "https://antenne.invalid/cnews");
        let groups: HashSet<_> = catalog.channels.iter().filter_map(|c| c.group.clone()).collect();
        let expected = ["National", "News", "Sport", "Thematic", "International", "Radio", "Regional", "Overseas", "Local"];
        assert_eq!(groups, expected.iter().map(ToString::to_string).collect());
    }

    #[test]
    fn the_builders_make_the_entries() {
        let by_id = |id| *find(id).unwrap();
        assert_eq!(
            tnt(2, ch("france-2", "France 2", "National", by_id("france-2").logo, ftv("006194ea-117d-4bcf-94a9-153d999c59ae"))),
            by_id("france-2")
        );
        assert_eq!(ch("cnews", "CNews", "News", by_id("cnews").logo, dm("x3b68jn")).source, by_id("cnews").source);
        assert_eq!(dm_in("x9jhhyc", "https://t18.fr/"), by_id("t18").source);
        assert_eq!(bfm_local("bfm-lyon", "BFM Lyon", "lyon/en-direct/"), by_id("bfm-lyon"));
        assert_eq!(france_3("france-3-noa", "France 3 NoA", "ea65a32f-11ed-11ea-8b6e-0050568b2441"), by_id("france-3-noa"));
        assert_eq!(equidia("equidia-live2"), by_id("equidia").source);
        assert_eq!(fixed("https://static.lefigaro.fr/secom/tnt.m3u8"), by_id("figaro-tv-idf").source);
    }

    #[test]
    fn channels_are_found_by_id() {
        assert_eq!(find("france-2").unwrap().source, Source::FranceTv { si_id: "006194ea-117d-4bcf-94a9-153d999c59ae" });
        assert_eq!(find("tvr").unwrap().source, Source::EasyBroadcast { tenant: "captaprod", event: "54_tvr_vtucuxt" });
        assert_eq!(find("nope"), None);
    }
}
