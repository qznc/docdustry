use crate::{config::Config, database::Database};
use log::{debug, info, warn};
use pulldown_cmark::{CowStr, Event, Parser, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
use rouille::Response;
use std::{collections::HashMap, fs::File, path::PathBuf};

pub(crate) fn cmd_serve(cfg: Config) {
    let addr = "0.0.0.0:8081";
    println!("Start webserver at http://{}", addr);
    rouille::start_server(addr, move |request| {
        info!("Request: {:?}", request);

        if request.method() != "GET" {
            return Response::empty_404();
        }
        let url = request.url();
        if let Some(sub_url) = url.strip_prefix("/_static/") {
            return static_file(&cfg.serve.theme, sub_url);
        }
        let did: &str = if request.url() == "/" {
            "default"
        } else {
            url.strip_prefix("/").unwrap()
        };
        if let Some(html) = render(&cfg, did) {
            Response::html(html)
        } else {
            debug!("Did not find page {}", did);
            Response::empty_404()
        }
    });
}

fn static_file(theme_dir: &PathBuf, url: &str) -> Response {
    let mut path = theme_dir.clone();
    path.push(url);
    let ext: &str = path.extension().unwrap().to_str().unwrap();
    debug!("Load {:?}", path);
    let file = File::open(&path).unwrap();
    let typ = match ext {
        "png" => "image/png",
        "jpg" => "image/jpg",
        "css" => "text/css",
        "js" => "text/javascript",
        _ => "text/html",
    };
    Response::from_file(typ, file)
}

const TEMPLATE: &str = "<!DOCTYPE html>
<html>
<head>
<title>TITLE</title>
<meta charset=\"utf-8\" />
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" />
<script src=\"_static/theme.js\"></script>
<link rel=\"stylesheet\" type=\"text/css\" href=\"_static/theme.css\" />
</head>
<body>
<header></header>
<main>CONTENT</main>
<footer></footer>
</body>
</html>";

fn render(cfg: &Config, did: &str) -> Option<String> {
    let db = match Database::open(&cfg) {
        Ok(x) => x,
        Err(e) => {
            debug!("Error: {}", e);
            return None;
        }
    };
    let dids = get_dids(did, db);
    debug!("DIDs needed: {:?}", dids.keys());
    let base = dids.get(did).unwrap();
    let content = markdown_to_html(base.raw.as_str(), &dids);
    Some(
        TEMPLATE
            .replace("CONTENT", &content)
            .replace("TITLE", base.title.as_str()),
    )
}

fn markdown_to_html(markdown: &str, dids: &HashMap<String, Doc>) -> String {
    let mut html = String::new();
    let mut parser = Parser::new(markdown);
    let mut including = false;
    while let Some(event) = parser.next() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => html.push_str("<p>"),
                Tag::Heading {
                    level,
                    id: _,
                    classes: _,
                    attrs: _,
                } => {
                    html.push_str("<");
                    html.push_str(&level.to_string());
                    html.push_str(">");
                }
                Tag::BlockQuote => html.push_str("<blockquote>"),
                Tag::CodeBlock(kind) => match kind {
                    pulldown_cmark::CodeBlockKind::Indented => html.push_str("<pre><code>"),
                    pulldown_cmark::CodeBlockKind::Fenced(language) => {
                        gen_codeblock(language.to_string().as_str(), &mut html, &mut parser);
                    }
                },
                Tag::HtmlBlock => html.push_str("<div>"),
                Tag::Item => html.push_str("<li>"),
                Tag::FootnoteDefinition(_) => html.push_str("<div class=\"footnote\">"),
                Tag::Table(_) => html.push_str("<table>"),
                Tag::List(first) => match first {
                    Some(start_num) => {
                        html.push_str("<ol");
                        if start_num != 1 {
                            html.push_str(&format!(r#" start="{}""#, start_num));
                        }
                        html.push('>');
                    }
                    None => html.push_str("<ul>"),
                },
                Tag::TableHead => html.push_str("<thead>"),
                Tag::TableRow => html.push_str("<tr>"),
                Tag::TableCell => html.push_str("<td>"),
                Tag::Emphasis => html.push_str("<em>"),
                Tag::Strong => html.push_str("<strong>"),
                Tag::Strikethrough => html.push_str("<del>"),
                Tag::Link {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    html.push_str("<a href=\"");
                    html.push_str(&dest_url);
                    html.push_str("\">");
                }
                Tag::Image {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    let durl = dest_url.to_string();
                    if dest_url.starts_with("did:") {
                        let inner_did = durl.strip_prefix("did:").unwrap();
                        debug!("include {}", inner_did);

                        if let Some(inner_md) = dids.get(&inner_did.to_string()) {
                            let inner_html = markdown_to_html(inner_md.raw.as_str(), dids);
                            html.push_str("<div class=\"inclusion\">");
                            html.push_str(inner_html.as_str());
                            html.push_str("</div>\n");
                            including = true;
                        } else {
                            warn!("Link to missing DID {}", inner_did);
                            html.push_str(format!("MISSING {}<br/>", inner_did).as_str());
                        }
                    } else {
                        html.push_str("<img src=\"");
                        html.push_str(&dest_url);
                        html.push_str("\" alt=\"image\">");
                    }
                }
                Tag::MetadataBlock(_) => (),
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => html.push_str("</p>\n"),
                TagEnd::Heading(level) => {
                    html.push_str("</");
                    html.push_str(&level.to_string());
                    html.push_str(">\n");
                }
                TagEnd::BlockQuote => html.push_str("</blockquote>\n"),
                TagEnd::CodeBlock => html.push_str("</code></pre>\n"),
                TagEnd::HtmlBlock => html.push_str("</div>\n"),
                TagEnd::List(ordered) => match ordered {
                    true => html.push_str("</ol>\n"),
                    false => html.push_str("</ul>\n"),
                },
                TagEnd::Item => html.push_str("</li>"),
                TagEnd::FootnoteDefinition => html.push_str("</div>\n"),
                TagEnd::Table => html.push_str("</table>\n"),
                TagEnd::TableHead => html.push_str("</thead>"),
                TagEnd::TableRow => html.push_str("</tr>"),
                TagEnd::TableCell => html.push_str("</td>"),
                TagEnd::Emphasis => html.push_str("</em>"),
                TagEnd::Strong => html.push_str("</strong>"),
                TagEnd::Strikethrough => html.push_str("</del>"),
                TagEnd::Link => html.push_str("</a>"),
                TagEnd::Image => {
                    if !including {
                        html.push_str("</img>\n")
                    }
                }
                TagEnd::MetadataBlock(_) => (),
            },
            Event::Text(t) => {
                if !including {
                    escape_html(&mut html, &t).unwrap()
                }
            }
            Event::Code(c) => {
                html.push_str("<code>");
                escape_html(&mut html, &c).unwrap();
                html.push_str("</code>");
            }
            Event::Html(h) => html.push_str(&h),
            Event::InlineHtml(h) => html.push_str(&h),
            Event::FootnoteReference(name) => {
                html.push_str("<sup><a href=\"#fn-");
                html.push_str(&name);
                html.push_str("\">");
                html.push_str(&name);
                html.push_str("</a></sup>\n");
            }
            Event::SoftBreak => html.push_str("\n"),
            Event::HardBreak => html.push_str("<br />\n"),
            Event::Rule => html.push_str("<hr />\n"),
            Event::TaskListMarker(checked) => {
                html.push_str("<input type=\"checkbox\" disabled");
                if checked {
                    html.push_str(" checked");
                }
                html.push_str(">");
            }
        }
    }
    html
}

fn gen_codeblock(language: &str, html: &mut String, parser: &mut Parser) {
    match language {
        // TODO handle special "languages"
        _ => html.push_str("<pre><code>"),
    }
}

struct Doc {
    raw: String,
    title: String,
}

fn get_dids(did: &str, db: Database) -> HashMap<String, Doc> {
    let mut dids_md: HashMap<String, Doc> = HashMap::new();
    let mut dids_todo: Vec<String> = vec![did.to_string()];
    while !dids_todo.is_empty() {
        let current = dids_todo.pop().unwrap();
        let raw = db.get_did(&current).unwrap();
        let mut doc = Doc {
            raw: raw.clone(),
            title: String::from(did),
        };
        let mut parser = Parser::new(raw.as_str());
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Image {
                        link_type: _,
                        dest_url,
                        title: _,
                        id: _,
                    } => {
                        let durl = dest_url.to_string();
                        if dest_url.starts_with("did:") {
                            let inner_did = durl.strip_prefix("did:").unwrap();
                            debug!("ref include {}", inner_did);
                            dids_todo.push(inner_did.to_string());
                        }
                    }
                    Tag::CodeBlock(kind) => match kind {
                        pulldown_cmark::CodeBlockKind::Indented => (),
                        pulldown_cmark::CodeBlockKind::Fenced(language) => {
                            if language == CowStr::from("docdustry-docmeta") {
                                if let Some(Event::Text(text)) = parser.next() {
                                    let t: String = text.to_string();
                                    for line in t.lines() {
                                        if let Some((k, v)) = line.split_once(":") {
                                            match k {
                                                "title" => doc.title = v.to_string(),
                                                _ => (),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => (),
                },
                _ => (),
            }
        }
        dids_md.insert(current.to_string(), doc);
    }
    dids_md
}
