use ignore::Walk;
use log::{error, info, warn};
use pulldown_cmark::Parser;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
use std::collections::{HashMap, VecDeque};
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct Doc {
    /// document ID, the unique identifier for linking to it
    pub did: String,
    pub title: String,
    pub status: String,
    pub links: Vec<String>,
    pub includes: Vec<String>,
    pub tags: Vec<String>,
    pub url: String,

    #[serde(skip)]
    raw: String,
    #[serde(skip)]
    html: String,
    #[serde(skip)]
    pub src_path_rel: PathBuf,
    #[serde(skip)]
    src_path_base: PathBuf,
}

impl Doc {
    fn new(src_path_base: &PathBuf, src_path_rel: PathBuf) -> Doc {
        Doc {
            src_path_rel,
            src_path_base: src_path_base.to_path_buf(),
            html: String::with_capacity(4000),
            title: String::new(),
            links: vec![],
            tags: vec![],
            did: String::new(),
            status: String::new(),
            url: String::new(),
            includes: vec![],
            raw: String::new(),
        }
    }

    fn gen_html(&mut self) -> Result<(), io::Error> {
        let file = self.src_path_base.join(&self.src_path_rel);
        let raw = read_to_string(file)?;
        self.parse_md(&raw, &None);
        if !self.includes.is_empty() {
            // need at least a second run for the inclusion
            self.html.clear();
            self.raw = raw;
        }
        Ok(())
    }

    fn parse_md(&mut self, raw: &String, include_map: &Option<HashMap<String, String>>) {
        let mut parser = Parser::new(raw);
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading {
                        level,
                        id,
                        classes: _,
                        attrs: _,
                    } => {
                        self.html.push('<');
                        self.html.push_str(&level.to_string());
                        if let Some(s) = id {
                            self.html.push_str(&" id=\"");
                            self.html.push_str(&s);
                            self.html.push('"');
                        }
                        self.html.push('>');
                        if level == HeadingLevel::H1 {
                            if let Some(Event::Text(t)) = parser.next() {
                                self.html.push_str(&t);
                                if self.title.is_empty() {
                                    self.title = t.to_string();
                                }
                            }
                        }
                    }
                    Tag::CodeBlock(kind) => match kind {
                        CodeBlockKind::Indented => self.html.push_str(&"<pre><code>"),
                        CodeBlockKind::Fenced(lang) => self.gen_codeblock(lang, &mut parser),
                    },
                    Tag::Link {
                        link_type: _,
                        dest_url,
                        title,
                        id,
                    } => self.gen_link(dest_url, id, title),
                    Tag::Paragraph => self.html.push_str(&"<p>"),
                    Tag::BlockQuote => self.html.push_str(&"<blockquote>"),
                    Tag::HtmlBlock => self.html.push_str(&"<div html>"),
                    Tag::List(first) => match first {
                        Some(start_num) => {
                            self.html.push_str(&"<ol");
                            if start_num != 1 {
                                self.html.push_str(&format!(r#" start="{}""#, start_num));
                            }
                            self.html.push('>');
                        }
                        None => self.html.push_str(&"<ul>"),
                    },
                    Tag::Item => self.html.push_str(&"<li>"),
                    Tag::FootnoteDefinition(_) => todo!(),
                    Tag::Table(_) => todo!(),
                    Tag::TableHead => self.html.push_str(&"<th>"),
                    Tag::TableRow => self.html.push_str(&"<tr>"),
                    Tag::TableCell => self.html.push_str(&"<td>"),
                    Tag::Emphasis => self.html.push_str(&"<em>"),
                    Tag::Strong => self.html.push_str(&"<strong>"),
                    Tag::Strikethrough => self.html.push_str(&"<del>"),
                    Tag::Image {
                        link_type: _,
                        dest_url,
                        title,
                        id,
                    } => self.gen_img(dest_url, include_map, &mut parser, id, title),
                    Tag::MetadataBlock(_) => todo!(),
                },
                Event::End(tag) => match tag {
                    TagEnd::Paragraph => self.html.push_str(&"</p>"),
                    TagEnd::Heading(level) => self.html.push_str(&format!("</{}>", level)),
                    TagEnd::BlockQuote => self.html.push_str(&"</blockquote>"),
                    TagEnd::CodeBlock => self.html.push_str(&"</pre></code>"),
                    TagEnd::HtmlBlock => self.html.push_str(&"</div>"),
                    TagEnd::List(ordered) => match ordered {
                        true => self.html.push_str(&"</ol>"),
                        false => self.html.push_str(&"</ul>"),
                    },
                    TagEnd::Item => self.html.push_str(&"</li>"),
                    TagEnd::FootnoteDefinition => todo!(),
                    TagEnd::Table => self.html.push_str(&"</table>"),
                    TagEnd::TableHead => self.html.push_str(&"</th>"),
                    TagEnd::TableRow => self.html.push_str(&"</tr>"),
                    TagEnd::TableCell => self.html.push_str(&"</td>"),
                    TagEnd::Emphasis => self.html.push_str(&"</em>"),
                    TagEnd::Strong => self.html.push_str(&"</strong>"),
                    TagEnd::Strikethrough => self.html.push_str(&"</del>"),
                    TagEnd::Link => self.html.push_str(&"</a>"),
                    TagEnd::Image => self.html.push_str(&"</img>"),
                    TagEnd::MetadataBlock(_) => todo!(),
                },
                Event::Text(t) => escape_html(&mut self.html, &t).unwrap(),
                Event::Code(c) => {
                    self.html.push_str(&"<code>");
                    escape_html(&mut self.html, &c).unwrap();
                    self.html.push_str(&"</code>");
                }
                Event::Html(t) => self.html.push_str(&t),
                Event::InlineHtml(t) => self.html.push_str(&t),
                Event::FootnoteReference(_) => todo!(),
                Event::SoftBreak => self.html.push('\n'),
                Event::HardBreak => self.html.push_str(&"<br/>"),
                Event::Rule => self.html.push_str(&"<hr/>"),
                Event::TaskListMarker(_) => todo!(),
            }
        }
        // post-processing
        if self.did.is_empty() {
            let mut ctx = md5::Context::new();
            ctx.consume(&self.src_path_rel.as_os_str().as_encoded_bytes());
            ctx.consume(&self.title);
            let hash = ctx.compute();
            self.did = format!("{:x}", hash);
        }
        if self.title.is_empty() {
            self.title.push_str(&"<unknown>");
        }
        self.url = self.rel_url();
    }

    fn gen_img(
        &mut self,
        dest_url: CowStr<'_>,
        include_map: &Option<HashMap<String, String>>,
        parser: &mut Parser<'_>,
        id: CowStr<'_>,
        title: CowStr<'_>,
    ) {
        if dest_url.starts_with(&"did:") {
            // include another page
            let did = dest_url[4..].to_string();
            match include_map {
                Some(m) => {
                    match m.get(&did) {
                        Some(html) => {
                            self.html.push_str(&r#"<article class="inclusion">"#);
                            self.html.push_str(&r#"<a class="inclusion" href=""#);
                            self.html.push_str(&dest_url);
                            self.html.push_str(&"\">inclusion</a>\n");
                            self.html.push_str(html);
                            self.html.push_str(&"</article>\n");
                        }
                        None => {
                            warn!("Including non-existant DID: {}", dest_url);
                            self.html.push_str(&r#"<p class="error">Inclusion fail: "#);
                            self.html.push_str(&dest_url);
                            self.html.push_str(&"</p>\n");
                        }
                    };
                    skip_img_rest(parser);
                }
                None => {
                    self.includes.push(did);
                }
            };
        } else {
            // normal image
            self.html.push_str(&"<img");
            if !dest_url.is_empty() {
                self.html.push_str(" src=\"");
                self.html.push_str(&dest_url);
                self.html.push('"');
            }
            if !id.is_empty() {
                self.html.push_str(" id=\"");
                self.html.push_str(&id);
                self.html.push('"');
            }
            if !title.is_empty() {
                self.html.push_str(" title=\"");
                self.html.push_str(&title);
                self.html.push('"');
            }
            self.html.push('>');
            skip_img_rest(parser);
        }
    }

    fn gen_link(&mut self, dest_url: CowStr<'_>, id: CowStr<'_>, title: CowStr<'_>) {
        self.html.push_str(&"<a href=\"");
        self.html.push_str(&dest_url);
        self.html.push('"');
        if !id.is_empty() {
            self.html.push_str(&" id=\"");
            self.html.push_str(&id);
            self.html.push('"');
        }
        if !title.is_empty() {
            self.html.push_str(&" title=\"");
            self.html.push_str(&title);
            self.html.push('"');
        }
        self.html.push('>');
        if dest_url.starts_with(&"https://")
            || dest_url.starts_with(&"http://")
            || dest_url.starts_with(&"#")
        {
        } else {
            self.links.push(dest_url.to_string());
        }
    }

    fn gen_codeblock(&mut self, lang: CowStr<'_>, parser: &mut Parser<'_>) {
        let meta: bool = lang == CowStr::from("docdustry-docmeta");
        if meta {
            self.html.push_str(&r#"<details class=\"metainfo">"#);
            self.html.push_str(&"<summary>doc meta info</summary>");
        }
        self.html.push_str(&"<pre class=\"language-");
        if lang.is_empty() {
            self.html.push_str(&"unknown");
        } else {
            self.html.push_str(&lang);
        }
        self.html.push_str(&"\"><code>");
        while let Some(event) = parser.next() {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    self.html.push_str(&"</code></pre>");
                    break;
                }
                Event::Text(t) => {
                    if meta {
                        self.parse_meta(t.to_string());
                    }
                    escape_html(&mut self.html, &t).unwrap();
                }
                _ => todo!(),
            }
        }
        if meta {
            self.html.push_str(&"</details>");
        }
    }

    fn parse_meta(&mut self, meta: String) {
        for line in meta.split("\n") {
            if let Some((k, v)) = line.split_once(":") {
                match k {
                    "status" => {
                        self.status = v.trim().to_string();
                    }
                    "id" => {
                        self.did = v.trim().to_string();
                    }
                    "tag" => {
                        self.tags.push(v.trim().to_string());
                    }
                    _ => (),
                }
            }
        }
    }

    pub(crate) fn write_html(&self, st: &mut BufWriter<File>) -> Result<usize, io::Error> {
        st.write(self.html.as_bytes())
    }

    pub fn shorthash(&self) -> String {
        let dir = self.src_path_rel.parent().unwrap();
        let hash = md5::compute(dir.as_os_str().as_encoded_bytes());
        let hex_string = format!("{:x}", hash);
        let short_hash = &hex_string[..6];
        short_hash.to_string()
    }

    pub fn rel_url(&self) -> String {
        let mut rel_url = PathBuf::new();
        rel_url.push("..");
        rel_url.push(self.html_path());
        rel_url.to_string_lossy().to_string()
    }

    pub(crate) fn html_path(&self) -> PathBuf {
        let mut out_path_rel = PathBuf::new();
        out_path_rel.push(self.shorthash());
        let basename = self
            .src_path_rel
            .file_stem()
            .expect("stem")
            .to_str()
            .expect("str");
        let out_file_name = format!("{}.html", basename);
        out_path_rel.push(out_file_name);
        out_path_rel
    }
}

fn skip_img_rest(parser: &mut Parser<'_>) {
    let n = parser.next();
    match n {
        Some(Event::Text(_)) => {
            let nn = parser.next();
            match nn {
                Some(Event::End(TagEnd::Image)) => (),
                _ => {
                    todo!("unforeseen: {:?}", n);
                }
            }
        }
        Some(Event::End(TagEnd::Image)) => (),
        _ => {
            todo!("unforeseen: {:?}", n);
        }
    }
}

struct HtmlConverter {
    docs: Vec<Doc>,
    id2index: HashMap<String, usize>,
    includes_docs: VecDeque<usize>,
}

impl HtmlConverter {
    pub fn new() -> HtmlConverter {
        HtmlConverter {
            docs: vec![],
            includes_docs: VecDeque::new(),
            id2index: HashMap::new(),
        }
    }

    pub fn read_md_files(&mut self, src_path_base: PathBuf) {
        self.collect_md_files(src_path_base);
        self.markdown2html();
        while !self.includes_docs.is_empty() {
            let i = self.includes_docs.pop_front().unwrap();
            let map = self.include_map_if_ready(&self.docs[i]);
            if map.is_none() {
                continue;
            }
            let ref mut d = self.docs[i];
            info!("Repeat HTML generation: {}", d.did);
            d.parse_md(&d.raw.clone(), &map);
        }
    }

    fn markdown2html(&mut self) {
        let mut i: usize = 0;
        for d in &mut self.docs {
            let result = d.gen_html();
            if let Err(e) = result {
                let path = d.src_path_base.join(&d.src_path_rel);
                warn!("skip {}: {}", path.display(), e);
                continue;
            }
            self.id2index.insert(d.did.clone(), i);
            if !d.includes.is_empty() {
                self.includes_docs.push_back(i);
            }
            i += 1;
        }
    }

    fn collect_md_files(&mut self, src_path_base: PathBuf) {
        for result in Walk::new(&src_path_base) {
            match result {
                Ok(entry) => {
                    let t = entry.file_type().expect("file type");
                    if t.is_dir() {
                        continue;
                    }
                    let p = entry.path();
                    match p.extension() {
                        Some(ext) => {
                            if ext != "md" {
                                continue;
                            }
                        }
                        None => continue,
                    };
                    let src_path_rel = entry
                        .path()
                        .strip_prefix(&src_path_base)
                        .expect("is prefix")
                        .to_path_buf();
                    let doc = Doc::new(&src_path_base, src_path_rel);
                    self.docs.push(doc);
                }
                Err(err) => error!("Not an entry: {}", err),
            }
        }
        info!("Found {} md files", self.docs.len());
    }

    /// if all documents include by d are done, return a hashmap of did->html
    fn include_map_if_ready(&self, d: &Doc) -> Option<HashMap<String, String>> {
        let mut map: HashMap<String, String> = HashMap::new();
        for did in &d.includes {
            let j = match self.id2index.get(did) {
                Some(index) => *index,
                None => {
                    continue;
                }
            };
            let ref d2 = self.docs[j];
            if d2.html.is_empty() {
                // the included file is not finished yet (includes something itself?)
                return None;
            }
            map.insert(did.clone(), d2.html.clone());
        }
        Some(map)
    }
}

pub fn read_md_files(src_path_base: PathBuf) -> Vec<Doc> {
    let mut conv = HtmlConverter::new();
    conv.read_md_files(src_path_base);
    conv.docs
}
