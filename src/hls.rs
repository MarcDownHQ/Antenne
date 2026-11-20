//! Dailymotion masters list a primary and a backup encoder side by side, on unrelated timelines: a player that takes
//! video from one and audio from the other plays out of sync or silent. The master is cut down to one encoder.

use crate::dailymotion::{kept_master, master_video};
use crate::host::Storage;
use crate::model::Media;

/// An answer on its way to the player: Dailymotion masters lose their second encoder, the rest passes untouched.
pub fn filter(storage: &impl Storage, url: &str, media: Media) -> Media {
    let Some(video) = master_video(url) else { return media };
    let relayed = String::from_utf8_lossy(&media.body);
    if !relayed.starts_with("#EXTM3U") {
        return media;
    }
    let kept = storage.get(&kept_master(video)).filter(|kept| max_bandwidth(kept) > max_bandwidth(&relayed));
    let master = kept.as_deref().unwrap_or(&relayed);
    Media { content_type: media.content_type.clone(), body: one_encoder(master, video).into_bytes() }
}

/// The value of `name` in an `#EXT-X-…:A=1,B="x,y"` tag line.
fn attribute<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = &line[line.find(':')? + 1..];
    while !rest.is_empty() {
        let eq = rest.find('=')?;
        let key = &rest[..eq];
        let after = &rest[eq + 1..];
        let (value, next) = if let Some(quoted) = after.strip_prefix('"') {
            let end = quoted.find('"')?;
            (&quoted[..end], quoted[end + 1..].strip_prefix(',').unwrap_or(""))
        } else {
            after.split_once(',').unwrap_or((after, ""))
        };
        if key == name {
            return Some(value);
        }
        rest = next;
    }
    None
}

fn max_bandwidth(master: &str) -> u64 {
    master
        .lines()
        .filter(|l| l.starts_with("#EXT-X-STREAM-INF:"))
        .filter_map(|l| attribute(l, "BANDWIDTH")?.parse().ok())
        .max()
        .unwrap_or(0)
}

/// The tallest variant of a master, in lines.
pub fn max_height(master: &str) -> u32 {
    master
        .lines()
        .filter(|l| l.starts_with("#EXT-X-STREAM-INF:"))
        .filter_map(|l| attribute(l, "RESOLUTION")?.split_once('x')?.1.parse().ok())
        .max()
        .unwrap_or(0)
}

/// The Dailymotion video an address serves from: the encoder it belongs to.
fn encoder_of(uri: &str) -> Option<&str> {
    let path = uri.split(['?', '#']).next()?;
    path.split('/').find(|s| s.len() >= 5 && s.starts_with(['x', 'k']) && s.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase()))
}

fn groups<'a>(variants: &[Variant<'a>], encoder: &str) -> Vec<&'a str> {
    variants.iter().filter(|v| v.encoder == encoder).filter_map(|v| attribute(v.inf, "AUDIO")).collect()
}

struct Variant<'a> {
    inf: &'a str,
    uri: &'a str,
    encoder: String,
}

/// Keeps one encoder: the video asked for, else the one not marked `-backup`, else the `fre` audio over `und`.
fn one_encoder(master: &str, video: &str) -> String {
    let lines: Vec<&str> = master.lines().collect();
    let mut variants = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("#EXT-X-STREAM-INF:") && i + 1 < lines.len() {
            let (inf, uri) = (lines[i], lines[i + 1]);
            let group = attribute(inf, "AUDIO").unwrap_or_default();
            let encoder =
                encoder_of(uri).map_or_else(|| if group.ends_with("-backup") { "backup" } else { "primary" }.to_string(), str::to_string);
            variants.push(Variant { inf, uri, encoder });
            i += 1;
        }
        i += 1;
    }
    let mut encoders: Vec<&str> = Vec::new();
    for v in &variants {
        if !encoders.contains(&v.encoder.as_str()) {
            encoders.push(&v.encoder);
        }
    }
    if encoders.len() < 2 {
        return master.to_string();
    }
    let audio_name = |group: &str| {
        lines.iter().find(|l| l.starts_with("#EXT-X-MEDIA:") && attribute(l, "GROUP-ID") == Some(group)).and_then(|l| attribute(l, "NAME"))
    };
    let backups = encoders.iter().any(|e| groups(&variants, e).iter().any(|g| g.ends_with("-backup")));
    let chosen = encoders
        .iter()
        .find(|e| **e == video)
        .or_else(|| encoders.iter().find(|e| backups && groups(&variants, e).iter().all(|g| !g.ends_with("-backup"))))
        .or_else(|| {
            encoders.iter().find(|e| {
                let g = groups(&variants, e);
                !g.is_empty() && g.iter().all(|g| audio_name(g) == Some("fre"))
            })
        })
        .unwrap_or(&encoders[0]);
    let kept_groups = groups(&variants, chosen);
    let dropped: Vec<&str> = variants.iter().filter(|v| v.encoder != *chosen).flat_map(|v| [v.inf, v.uri]).collect();
    let mut out = String::with_capacity(master.len());
    for line in &lines {
        let unused_audio = line.starts_with("#EXT-X-MEDIA:") && attribute(line, "GROUP-ID").is_some_and(|g| !kept_groups.contains(&g));
        if unused_audio || dropped.iter().any(|d| std::ptr::eq(*d, *line)) {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeStorage, fixture};

    const CNEWS: &str = "https://cdndirector.dailymotion.com/cdn/live/video/x3b68jn.m3u8?sec=SIGNED";

    fn filtered(storage: &FakeStorage, url: &str, body: &str) -> String {
        let media = filter(storage, url, Media { content_type: Some("application/x-mpegURL".into()), body: body.as_bytes().to_vec() });
        assert_eq!(media.content_type.as_deref(), Some("application/x-mpegURL"));
        String::from_utf8(media.body).unwrap()
    }

    fn uris(master: &str) -> Vec<&str> {
        master.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect()
    }

    #[test]
    fn keeps_the_primary_encoder_and_its_audio() {
        let out = filtered(&FakeStorage::new(), CNEWS, &fixture("dm-master-cnews.m3u8"));
        assert_eq!(uris(&out).len(), 5);
        assert!(uris(&out).iter().all(|u| u.contains("/x3b68jn/")));
        assert!(!out.contains("x5idxxp") && !out.contains("-backup"));
        assert_eq!(out.matches("TYPE=AUDIO").count(), 2);
        assert!(out.starts_with("#EXTM3U\n#EXT-X-VERSION:7\n"));
    }

    #[test]
    fn keeps_the_requested_video_of_a_muxed_master() {
        let out = filtered(
            &FakeStorage::new(),
            "https://cdndirector.dailymotion.com/cdn/live/video/xxtuy6.m3u8?sec=S",
            &fixture("dm-master-fun-radio.m3u8"),
        );
        assert_eq!(uris(&out).len(), 5);
        assert!(!out.contains("x7rk9ec"));
    }

    #[test]
    fn a_master_served_from_another_video_keeps_its_non_backup_encoder() {
        let rtl2 = fixture("dm-master-rtl2.m3u8");
        let out = filtered(&FakeStorage::new(), "https://cdndirector.dailymotion.com/cdn/live/video/xother.m3u8", &rtl2);
        assert!(uris(&out).iter().all(|u| u.contains("/x2tzzpj/")));
        assert_eq!(out.matches("TYPE=AUDIO").count(), 2);
    }

    #[test]
    fn two_encoders_without_suffix_keep_the_fre_audio() {
        let master = "#EXTM3U\n\
            #EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"a\",NAME=\"und\",URI=\"https://l.test/s/xaaaa1/d/aac.m3u8\"\n\
            #EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"b\",NAME=\"fre\",URI=\"https://l.test/s/xbbbb2/d/aac.m3u8\"\n\
            #EXT-X-STREAM-INF:BANDWIDTH=1,AUDIO=\"a\"\nhttps://l.test/s/xaaaa1/d/v.m3u8\n\
            #EXT-X-STREAM-INF:BANDWIDTH=1,AUDIO=\"b\"\nhttps://l.test/s/xbbbb2/d/v.m3u8\n";
        let out = filtered(&FakeStorage::new(), "https://cdndirector.dailymotion.com/cdn/live/video/xcccc3.m3u8", master);
        assert_eq!(uris(&out), ["https://l.test/s/xbbbb2/d/v.m3u8"]);
        assert!(!out.contains("\"und\""));
    }

    #[test]
    fn otherwise_the_first_encoder_stays() {
        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nhttps://l.test/xaaaa1/v.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=1\nhttps://l.test/xbbbb2/v.m3u8\n";
        let out = filtered(&FakeStorage::new(), "https://cdndirector.dailymotion.com/cdn/live/video/xcccc3.m3u8", master);
        assert_eq!(uris(&out), ["https://l.test/xaaaa1/v.m3u8"]);
    }

    #[test]
    fn backup_groups_tell_encoders_apart_when_addresses_do_not() {
        let master = "#EXTM3U\n\
            #EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"g\",URI=\"a1.m3u8\"\n\
            #EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"g-backup\",URI=\"a2.m3u8\"\n\
            #EXT-X-STREAM-INF:BANDWIDTH=1,AUDIO=\"g-backup\"\nv2.m3u8\n\
            #EXT-X-STREAM-INF:BANDWIDTH=1,AUDIO=\"g\"\nv1.m3u8\n";
        let out = filtered(&FakeStorage::new(), "https://cdndirector.dailymotion.com/cdn/live/video/xcccc3.m3u8", master);
        assert_eq!(uris(&out), ["v1.m3u8"]);
        assert!(out.contains("a1.m3u8") && !out.contains("a2.m3u8"));
    }

    #[test]
    fn a_single_encoder_passes_as_is() {
        for name in ["dm-master-france-inter.m3u8", "dm-master-t18.m3u8"] {
            let master = fixture(name);
            assert_eq!(filtered(&FakeStorage::new(), CNEWS, &master), master, "{name}");
        }
    }

    #[test]
    fn a_fuller_kept_copy_wins_over_a_truncated_ladder() {
        let full = fixture("dm-master-cnews.m3u8");
        let short = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=475136\nhttps://l.test/x3b68jn/240.m3u8\n";
        let storage = FakeStorage::new().with("dm:master:x3b68jn", &full);
        assert_eq!(uris(&filtered(&storage, CNEWS, short)).len(), 5);
        let storage = FakeStorage::new().with("dm:master:x3b68jn", short);
        assert_eq!(uris(&filtered(&storage, CNEWS, &full)).len(), 5);
    }

    #[test]
    fn everything_else_passes_untouched() {
        let storage = FakeStorage::new();
        let segment = Media { content_type: Some("video/iso.segment".into()), body: vec![0, 1, 2] };
        assert_eq!(filter(&storage, "https://live.cf.dmcdn.net/sec2(S)/dm/3/x3b68jn/d/live-1.m4s", segment.clone()), segment);
        let html = Media { content_type: Some("text/html".into()), body: b"<html>".to_vec() };
        assert_eq!(filter(&storage, CNEWS, html.clone()), html);
    }

    #[test]
    fn reads_tag_attributes() {
        let line = r#"#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="a,b",NAME="fre",DEFAULT=YES"#;
        assert_eq!(attribute(line, "GROUP-ID"), Some("a,b"));
        assert_eq!(attribute(line, "NAME"), Some("fre"));
        assert_eq!(attribute(line, "DEFAULT"), Some("YES"));
        assert_eq!(attribute(line, "URI"), None);
        assert_eq!(attribute("#EXTM3U", "A"), None);
        assert_eq!(attribute("#X:A", "A"), None);
        assert_eq!(attribute("#X:A=\"open", "A"), None);
        assert_eq!(max_bandwidth("#EXT-X-STREAM-INF:BANDWIDTH=x\nv\n"), 0);
    }

    #[test]
    fn measures_the_tallest_variant() {
        assert_eq!(max_height(&fixture("dm-master-cnews.m3u8")), 1080);
        assert_eq!(max_height(&fixture("dm-master-t18.m3u8")), 477);
        assert_eq!(max_height("#EXT-X-STREAM-INF:RESOLUTION=wide\nv\n#EXT-X-STREAM-INF:BANDWIDTH=1\nv\n"), 0);
    }

    #[test]
    fn encoders_are_the_video_ids_in_the_path() {
        assert_eq!(encoder_of("https://live.cf.dmcdn.net/sec2(T)/dm/3/x5idxxp/s/live-aac-64.m3u8#cell=a"), Some("x5idxxp"));
        assert_eq!(encoder_of("https://l.test/k1a2b3c/v.m3u8"), Some("k1a2b3c"));
        assert_eq!(encoder_of("v1.m3u8"), None);
    }
}
