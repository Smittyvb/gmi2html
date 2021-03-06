use gemtext::Node;

fn entity_escape_char(khar: &char) -> String {
    format!("&#x{:X};", (*khar) as u32)
}

fn html_escape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for khar in text.chars() {
        match khar {
            c @ '0'..='9' => result.push(c),
            c @ 'A'..='z' => result.push(c),
            ' ' => result.push(' '),
            c => result.push_str(&entity_escape_char(&c)),
        }
    }
    result
}

/// Converts the given Gemtext to the returned HTML.
pub fn gmi2html(gemtext: &str) -> String {
    nodes2html(gemtext::parse(gemtext))
}

/// Converts the Gemtext nodes into HTML.
pub fn nodes2html(nodes: Vec<Node>) -> String {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum MultilineState {
        List,
        Quote,
        None,
    }
    let mut html = String::new();
    let mut multiline_state = MultilineState::None;
    for node in nodes {
        let next_multiline_state = match node {
            Node::ListItem(_) => MultilineState::List,
            Node::Quote(_) => MultilineState::Quote,
            _ => MultilineState::None,
        };
        if next_multiline_state != multiline_state {
            // add closing tag
            html.push_str(match multiline_state {
                MultilineState::List => "</ul>",
                MultilineState::Quote => "</blockquote>",
                MultilineState::None => "",
            });
            // add opening tag
            html.push_str(match next_multiline_state {
                MultilineState::List => "<ul>",
                MultilineState::Quote => "<blockquote>",
                MultilineState::None => "",
            });
        };
        multiline_state = next_multiline_state;
        html.push_str(&match node {
            Node::ListItem(text) => format!("<li>{}</li>", html_escape(&text)),
            Node::Text(text) => format!("<p>{}</p>", html_escape(&text)),
            Node::Preformatted(text) => format!("<pre>{}</pre>", html_escape(&text)),
            Node::Heading { level, body } => format!(
                "<h{level}>{body}</h{level}>",
                level = level,
                body = html_escape(&body)
            ),
            Node::Quote(text) => format!("<blockquote>{}</blockquote>", html_escape(&text)),
            Node::Link { to, name } => {
                let name = name.unwrap_or_else(|| to.clone());
                format!(
                    r#"<p><a href="{}">{}</a></p>"#,
                    html_escape(&to),
                    html_escape(&name)
                )
            }
        });
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_list_tags() {
        let html = gmi2html(
            "line 1
* line 2
* line 3
line 4
* line 5
line 6",
        );
        assert_eq!(html, "<p>line 1</p><ul><li>line 2</li><li>line 3</li></ul><p>line 4</p><ul><li>line 5</li></ul><p>line 6</p>".to_string())
    }
}
