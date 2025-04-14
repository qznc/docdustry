use crate::{config::Config, database::Database};
use log::{debug, info, warn};
use pulldown_cmark::{CowStr, Event, Parser, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
use rouille::Response;
use std::collections::HashMap;

pub(crate) fn cmd_serve(cfg: Config) {
    let addr = "0.0.0.0:8081";
    println!("Start webserver at http://{}", addr);
    rouille::start_server(addr, move |request| {
        info!("Request: {:?}", request);

        if request.method() != "GET" {
            return Response::empty_404();
        }
        let url = request.url();
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
    for k in dids.keys() {
        let v = dids.get(k).unwrap();
        debug!("did {} -> {}", k, v);
    }
    let base = dids.get(did).unwrap();
    Some(markdown_to_html(base, &dids))
}

fn markdown_to_html(markdown: &str, dids: &HashMap<String, String>) -> String {
    let mut html = String::new();
    let mut parser = Parser::new(markdown);
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
                Tag::CodeBlock(_) => html.push_str("<pre><code>"),
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
                    link_type,
                    dest_url,
                    title,
                    id,
                } => {
                    let durl = dest_url.to_string();
                    if dest_url.starts_with("did:") {
                        let inner_did = durl.strip_prefix("did:").unwrap();
                        debug!("include {}", inner_did);

                        if let Some(inner_md) = dids.get(&inner_did.to_string()) {
                            let inner_html = markdown_to_html(inner_md, dids);
                            html.push_str(inner_html.as_str());
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
                TagEnd::Paragraph => html.push_str("</p>"),
                TagEnd::Heading(level) => {
                    html.push_str("</");
                    html.push_str(&level.to_string());
                    html.push_str(">");
                }
                TagEnd::BlockQuote => html.push_str("</blockquote>"),
                TagEnd::CodeBlock => html.push_str("</code></pre>"),
                TagEnd::HtmlBlock => html.push_str("</div>"),
                TagEnd::List(_) => html.push_str("</ul>"),
                TagEnd::Item => html.push_str("</li>"),
                TagEnd::FootnoteDefinition => html.push_str("</div>"),
                TagEnd::Table => html.push_str("</table>"),
                TagEnd::TableHead => html.push_str("</thead>"),
                TagEnd::TableRow => html.push_str("</tr>"),
                TagEnd::TableCell => html.push_str("</td>"),
                TagEnd::Emphasis => html.push_str("</em>"),
                TagEnd::Strong => html.push_str("</strong>"),
                TagEnd::Strikethrough => html.push_str("</del>"),
                TagEnd::Link => html.push_str("</a>"),
                TagEnd::Image => (),
                TagEnd::MetadataBlock(_) => (),
            },
            Event::Text(t) => escape_html(&mut html, &t).unwrap(),
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
                html.push_str("</a></sup>");
            }
            Event::SoftBreak => html.push_str("<br />"),
            Event::HardBreak => html.push_str("<br />"),
            Event::Rule => html.push_str("<hr />"),
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

fn get_dids(did: &str, db: Database) -> HashMap<String, String> {
    let mut dids_md: HashMap<String, String> = HashMap::new();
    let mut dids_todo: Vec<String> = vec![did.to_string()];
    while !dids_todo.is_empty() {
        let current = dids_todo.pop().unwrap();
        let raw = db.get_did(&current).unwrap();
        dids_md.insert(current.to_string(), raw.clone());
        let mut parser = Parser::new(raw.as_str());
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Image {
                        link_type,
                        dest_url,
                        title,
                        id,
                    } => {
                        let durl = dest_url.to_string();
                        if dest_url.starts_with("did:") {
                            let inner_did = durl.strip_prefix("did:").unwrap();
                            debug!("ref include {}", inner_did);
                            let inner_md = db.get_did(&inner_did.to_string()).unwrap();
                            dids_md.insert(inner_did.to_string(), inner_md);
                            dids_todo.push(inner_did.to_string());
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
    dids_md
}
