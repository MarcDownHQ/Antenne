//! Just enough HTML reading to lift a value out of a player page.

/// The value of attribute `name` in the first tag that contains `marker`, entities `&amp;` and `&quot;` decoded.
pub fn attribute(html: &str, marker: &str, name: &str) -> Option<String> {
    let at = html.find(marker)?;
    let start = html[..at].rfind('<')?;
    let end = at + html[at..].find('>')?;
    let tag = &html[start..end];
    let after = tag.find(&format!(" {name}="))? + name.len() + 2;
    let quote = tag[after..].chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let value = &tag[after + 1..];
    let value = &value[..value.find(quote)?];
    Some(value.replace("&quot;", "\"").replace("&amp;", "&"))
}

/// The text between the first `start` and the next `end`.
pub fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}

/// `https://host` of an address.
pub fn origin(url: &str) -> Option<&str> {
    let scheme = url.find("://")? + 3;
    let end = url[scheme..].find('/').map_or(url.len(), |i| scheme + i);
    Some(&url[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_attribute_of_the_marked_tag() {
        let html = r#"<p data-x="no"></p><div id="a" class="player box" data-url="/v.php?a=1&amp;b=2" data-q='it''s'>"#;
        assert_eq!(attribute(html, "player box", "data-url").as_deref(), Some("/v.php?a=1&b=2"));
        assert_eq!(attribute(html, "player box", "data-q").as_deref(), Some("it"));
        assert_eq!(attribute(html, "player box", "data-x"), None);
        assert_eq!(attribute(html, "missing", "data-url"), None);
        assert_eq!(attribute(r#"<a b=c d="e">"#, "d=", "b"), None);
        assert_eq!(attribute("player>", "player", "data-url"), None);
        assert_eq!(attribute(r#"<a player data-url="open>"#, "player", "data-url"), None);
        assert_eq!(attribute("<a player data-url=&quot;x&quot;>", "player", "data-url"), None);
    }

    #[test]
    fn reads_between_markers() {
        assert_eq!(between("a[b]c", "[", "]"), Some("b"));
        assert_eq!(between("a[b", "[", "]"), None);
        assert_eq!(between("ab", "[", "]"), None);
    }

    #[test]
    fn origins() {
        assert_eq!(origin("https://www.tv7.fr/a/b?c"), Some("https://www.tv7.fr"));
        assert_eq!(origin("https://www.tv7.fr"), Some("https://www.tv7.fr"));
        assert_eq!(origin("tv7.fr"), None);
    }
}
