use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::collections::HashSet;
use uuid::Uuid;

/// Parse markdown and extract metadata
pub struct MarkdownParser {
    wiki_link_pattern: Regex,
    tag_pattern: Regex,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            wiki_link_pattern: Regex::new(r"\[\[([^\]]+)\]\]").unwrap(),
            tag_pattern: Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap(),
        }
    }

    /// Convert markdown to HTML
    pub fn to_html(&self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(markdown, options);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }

    /// Extract wiki-style links from markdown [[Note Title]]
    pub fn extract_wiki_links(&self, markdown: &str) -> Vec<String> {
        self.wiki_link_pattern
            .captures_iter(markdown)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Extract hashtags from markdown
    pub fn extract_tags(&self, markdown: &str) -> Vec<String> {
        let tags: HashSet<String> = self
            .tag_pattern
            .captures_iter(markdown)
            .map(|cap| cap[1].to_string())
            .collect();

        tags.into_iter().collect()
    }

    /// Replace wiki links with HTML links
    pub fn process_wiki_links(&self, markdown: &str, note_resolver: impl Fn(&str) -> Option<Uuid>) -> String {
        self.wiki_link_pattern
            .replace_all(markdown, |caps: &regex::Captures| {
                let title = &caps[1];
                if let Some(note_id) = note_resolver(title) {
                    format!("[{}](note://{})", title, note_id)
                } else {
                    format!("[{}](create://{})", title, title)
                }
            })
            .to_string()
    }

    /// Extract metadata from frontmatter (YAML-like)
    pub fn extract_frontmatter(&self, markdown: &str) -> Option<Frontmatter> {
        if !markdown.starts_with("---") {
            return None;
        }

        let parts: Vec<&str> = markdown.splitn(3, "---").collect();
        if parts.len() < 3 {
            return None;
        }

        let frontmatter_text = parts[1].trim();
        let mut frontmatter = Frontmatter::default();

        for line in frontmatter_text.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "title" => frontmatter.title = Some(value.to_string()),
                    "tags" => {
                        frontmatter.tags = value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    "created" => frontmatter.created = Some(value.to_string()),
                    "updated" => frontmatter.updated = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Some(frontmatter)
    }

    /// Get content without frontmatter
    pub fn strip_frontmatter(&self, markdown: &str) -> String {
        if !markdown.starts_with("---") {
            return markdown.to_string();
        }

        let parts: Vec<&str> = markdown.splitn(3, "---").collect();
        if parts.len() < 3 {
            return markdown.to_string();
        }

        parts[2].trim().to_string()
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_links() {
        let parser = MarkdownParser::new();
        let markdown = "This is a [[Test Note]] and another [[Another Note]].";
        let links = parser.extract_wiki_links(markdown);

        assert_eq!(links.len(), 2);
        assert!(links.contains(&"Test Note".to_string()));
        assert!(links.contains(&"Another Note".to_string()));
    }

    #[test]
    fn test_extract_tags() {
        let parser = MarkdownParser::new();
        let markdown = "This is #tag1 and #tag2 content.";
        let tags = parser.extract_tags(markdown);

        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_to_html() {
        let parser = MarkdownParser::new();
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = parser.to_html(markdown);

        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }
}
