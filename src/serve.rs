use crate::{config::Config, database::Database};
use log::{debug, error, info, warn};
use pulldown_cmark::{CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
use rouille::Response;
use std::{fs::File, path::PathBuf};

pub fn cmd_serve(cfg: Config) {
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
        let db = match Database::open(&cfg) {
            Ok(x) => x,
            Err(e) => {
                debug!("Error: {}", e);
                return Response::empty_204();
            }
        };
        if request.url() == "/s" {
            let query = request.raw_query_string();
            return search_results(query, &db);
        }
        let did: &str = if request.url() == "/" {
            cfg.serve.frontpage.as_deref().unwrap_or("index")
        } else {
            url.strip_prefix("/").unwrap()
        };
        if let Some(html) = render(did, &db) {
            Response::html(html)
        } else {
            debug!("Did not find page {}", did);
            Response::empty_404()
        }
    });
}

fn search_results(query: &str, db: &Database) -> Response {
    if let Some(search_term) = parse_query(query) {
        let title = format!("Search Results: {}", search_term);
        let mut content = String::from("<h1>");
        content.push_str(&title);
        content.push_str("</h1>\n");
        let results = db.search(search_term);
        content.push_str("<ul class=\"search_results\">\n");
        for res in results {
            content.push_str("<li><a href=\"");
            content.push_str(&res.did);
            content.push_str("\">");
            content.push_str(&res.title);
            content.push_str("</a></li>\n");
        }
        content.push_str("</ul>\n");
        let html = render_template(&content, "", "", &title);
        Response::html(html)
    } else {
        Response::text("No search term")
    }
}

pub fn parse_query(query: &str) -> Option<&str> {
    for kv in query.split("&") {
        if let Some((k, v)) = kv.split_once("=") {
            if k == "s" {
                return Some(v);
            }
        }
    }
    None
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

fn render(did: &str, db: &Database) -> Option<String> {
    let md = db.get_did(did)?;
    let content = markdown_to_html(md.as_str(), &db);
    let meta = parse_meta_from_markdown(did, md.as_str());
    let backlinks = render_backlinks(did, &db);
    let relations = render_relations(did, &db);
    Some(render_template(
        &content,
        &backlinks,
        &relations,
        &meta.title,
    ))
}

fn render_template(content: &str, backlinks: &str, relations: &str, title: &str) -> String {
    let mut html = String::from("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head><title>");
    html.push_str(&title);
    html.push_str("</title>\n");
    html.push_str(
        "<meta charset=\"utf-8\" />
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" />
<meta name=\"color-scheme\" content=\"light dark\">
<script src=\"_static/theme.js\"></script>
<link rel=\"stylesheet\" type=\"text/css\" href=\"_static/theme.css\" />
</head>",
    );
    html.push_str("<body><header></header><div id=\"center\">");
    html.push_str("<main>");
    html.push_str(&content);
    html.push_str("</main>\n");
    html.push_str(
        "<div id=\"search\"><form method=\"GET\" action=\"s\">
        <input name=\"s\" placeholder=\"search term\" required />
        <button>Search</button>
    </form></div>",
    );
    html.push_str("<div id=\"relations\">");
    html.push_str(&relations);
    html.push_str("</div></div>\n<footer><div id=\"backlinks\">");
    html.push_str(&backlinks);
    html.push_str("</div></footer>\n");
    html.push_str(
        r##"<script type="module">
        if (document.querySelector('.mermaid')) {
          var mermaid = await import('https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs');
          mermaid.default.initialize({ startOnLoad: true });
        }
    </script>"##,
    );
    html.push_str("</body></html>");
    html
}

fn render_relations(did: &str, db: &Database) -> String {
    let mut ret = String::new();
    // Incoming
    let incoming = db.relations_with_tgt(did);
    if incoming.is_empty() {
        return ret;
    }
    ret.push_str("<ul class=\"incoming\">");
    for rel in incoming {
        ret.push_str("<li><a href=\"");
        ret.push_str(&rel.src);
        ret.push_str("\">");
        ret.push_str(&rel.src);
        ret.push_str("</a> ");
        ret.push_str(&rel.verb);
        ret.push_str(" this</li>");
    }
    ret.push_str("</ul>");
    // Outgoing
    let outgoing = db.relations_with_src(did);
    if outgoing.is_empty() {
        return ret;
    }
    ret.push_str("<ul class=\"outgoing\">");
    for rel in outgoing {
        ret.push_str("<li>this ");
        ret.push_str(&rel.verb);
        ret.push_str(" <a href=\"");
        ret.push_str(&rel.tgt);
        ret.push_str("\">");
        ret.push_str(&rel.tgt);
        ret.push_str("</a>");
        ret.push_str("</li>");
    }
    ret.push_str("</ul>");
    ret
}

fn render_backlinks(did: &str, db: &Database) -> String {
    let mut ret = String::new();
    let links = db.backlinks(did);
    if links.is_empty() {
        return ret;
    }
    ret.push_str("<ul>");
    for link in links {
        ret.push_str("<li><a href=\"");
        ret.push_str(&link);
        ret.push_str("\">");
        ret.push_str(&link);
        ret.push_str("</a></li>");
    }
    ret.push_str("</ul>");
    return ret;
}

fn markdown_to_html(markdown: &str, db: &Database) -> String {
    let mut html = String::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut parser = Parser::new_ext(markdown, options);
    // for img, remember if we are including a DID
    let mut including = false;
    // for links to DIDs, remember title in case the text is empty:
    let mut title_from_did = String::new();
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
                        gen_codeblock(language.to_string().as_str(), &mut html, &mut parser, &db);
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
                    link_type,
                    dest_url,
                    title,
                    id,
                } => {
                    title_from_did = to_html_link(&mut html, link_type, &dest_url, &title, &id);
                }
                Tag::Image {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    let durl = dest_url.to_string();
                    if dest_url.starts_with("did:") {
                        including = true;
                        let inner_did = durl.strip_prefix("did:").unwrap();
                        debug!("include {}", inner_did);
                        if let Some(inner_md) = db.get_did(&inner_did.to_string()) {
                            let inner_html = markdown_to_html(inner_md.as_str(), db);
                            html.push_str(
                                "<div class=\"inclusion\"><a class=\"inclusion\" href=\"",
                            );
                            html.push_str(inner_did);
                            html.push_str("\">inclusion</a>");

                            html.push_str(inner_html.as_str());
                            html.push_str("</div>\n");
                        } else {
                            warn!("Link to missing DID {}", inner_did);
                            html.push_str(format!("MISSING {}<br/>", inner_did).as_str());
                        }
                    } else {
                        including = false;
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
                TagEnd::Link => {
                    if !title_from_did.is_empty() {
                        let md = match db.get_did(&title_from_did) {
                            Some(x) => x,
                            None => {
                                error!("Document {} not found", title_from_did);
                                String::new()
                            }
                        };
                        let meta = parse_meta_from_markdown(title_from_did.as_str(), md.as_str());
                        html.push_str(&meta.title);
                        title_from_did.clear();
                    }
                    html.push_str("</a>");
                }
                TagEnd::Image => {
                    if including {
                        including = false;
                    } else {
                        html.push_str("</img>\n")
                    }
                }
                TagEnd::MetadataBlock(_) => (),
            },
            Event::Text(t) => {
                if including {
                    // skip image text during inclusion
                } else {
                    escape_html(&mut html, &t).unwrap()
                }
                title_from_did.clear(); // no need at TagEnd::Link anymore
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

fn to_html_link(
    html: &mut String,
    _link_type: LinkType,
    dest_url: &str,
    title: &str,
    id: &str,
) -> String {
    let mut title_from_did = String::new();
    html.push_str("<a href=\"");
    if let Some(did) = dest_url.strip_prefix("did:") {
        html.push_str(&did);
        title_from_did.insert_str(0, did);
    } else {
        html.push_str(&dest_url);
    }
    html.push('"');
    if !id.is_empty() {
        html.push_str(&" id=\"");
        html.push_str(&id);
        html.push('"');
    }
    if !title.is_empty() {
        html.push_str(&" title=\"");
        html.push_str(&title);
        html.push('"');
    }
    html.push('>');
    title_from_did
}

fn gen_codeblock(language: &str, html: &mut String, parser: &mut Parser, db: &Database) {
    match language {
        "docdustry-docmeta" => {
            html.push_str(&"<details class=\"metainfo\">");
            html.push_str(&"<summary>doc meta info</summary>");
            html.push_str(&"<pre class=\"docdustry-docmeta\"><code>");
            let text = skip_codeblock(parser);
            escape_html(&mut *html, &text).unwrap();
            html.push_str(&"</code></pre></details>\n");
        }
        "docdustry-mermaid" => {
            let text = skip_codeblock(parser);
            html.push_str(&"<pre class=\"mermaid\">");
            html.push_str(&text);
            html.push_str(&"</pre>\n");
        }
        "docdustry-doclist" => {
            let text = skip_codeblock(parser);
            if let Some((k, v)) = text.trim().split_once(":") {
                if k == "only-if-tagged" {
                    let docs = db.by_tag(v);
                    if docs.len() > 0 {
                        html.push_str("<ul class=\"doclist\">");
                        for md in docs {
                            let meta = parse_meta_from_markdown("", md.as_str());
                            html.push_str("<li><a href=\"");
                            html.push_str(&meta.did);
                            html.push_str("\">");
                            html.push_str(&meta.title);
                            html.push_str("</a></li>\n");
                        }
                        html.push_str("</ul>\n");
                    } else {
                        html.push_str("<p>Empty doclist</p>\n");
                    }
                } else {
                    html.push_str("<p>Unrestricted doclist</p>\n");
                }
            } else {
                html.push_str("<p>Very unrestricted doclist</p>\n");
            }
        }
        _ => html.push_str("<pre><code>"),
    }
}

fn skip_codeblock(parser: &mut Parser) -> String {
    let mut text = String::new();
    while let Some(event) = parser.next() {
        match event {
            Event::End(TagEnd::CodeBlock) => {
                break;
            }
            Event::Text(t) => {
                text.push_str(&t);
            }
            _ => todo!(),
        }
    }
    text
}

struct Doc {
    title: String,
    did: String,
    tags: Vec<String>,
}

fn parse_meta_from_markdown(did: &str, markdown: &str) -> Doc {
    let mut doc = Doc {
        title: String::from(did),
        did: String::from(did),
        tags: vec![],
    };
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let mut parser = Parser::new_ext(markdown, options);
    while let Some(event) = parser.next() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    if level == HeadingLevel::H1 {
                        if let Some(Event::Text(t)) = parser.next() {
                            doc.title = t.to_string();
                        }
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
                                        //debug!("docmeta {} => {}", k, v);
                                        match k {
                                            "id" => doc.did = v.trim().to_string(),
                                            "title" => doc.title = v.trim().to_string(),
                                            "tag" => doc.tags.push(v.trim().to_string()),
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
    doc
}
