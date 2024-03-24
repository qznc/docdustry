use ignore::Walk;
use log::{error, info};
use pulldown_cmark::Parser;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};
use pulldown_cmark_escape::escape_html;
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
    pub tags: Vec<String>,
    pub url: String,

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
        }
    }

    fn gen_html(&mut self) -> Result<(), io::Error> {
        let file = self.src_path_base.join(&self.src_path_rel);
        let markdown_content = read_to_string(file)?;
        self.parse_md(Parser::new(&markdown_content));
        // TODO: for inlining other pages we to track dependencies
        Ok(())
    }

    fn parse_md(&mut self, mut parser: Parser) {
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
                        CodeBlockKind::Fenced(lang) => {
                            self.html.push_str(&"<pre><code>");
                            if lang != CowStr::from("docdustry-docmeta") {
                                continue;
                            }
                            if let Some(Event::Text(t)) = parser.next() {
                                self.parse_meta(t.to_string());
                                escape_html(&mut self.html, &t).unwrap();
                            } else {
                                todo!();
                            }
                        }
                    },
                    Tag::Link {
                        link_type: _,
                        dest_url,
                        title,
                        id,
                    } => {
                        self.html.push_str(&format!(r#"<a href="{}">"#, dest_url));
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
                        if dest_url.starts_with(&"https://")
                            || dest_url.starts_with(&"http://")
                            || dest_url.starts_with(&"#")
                        {
                            continue;
                        }
                        self.links.push(dest_url.to_string());
                    }
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
                        link_type,
                        dest_url,
                        title,
                        id,
                    } => self
                        .html
                        .push_str(&format!(r#"<img src="{}" title="{}">"#, dest_url, title)),
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
                    TagEnd::Image => (),
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

struct HtmlConverter {
    docs: Vec<Doc>,
}

impl HtmlConverter {
    pub fn new() -> HtmlConverter {
        HtmlConverter { docs: vec![] }
    }

    pub fn read_md_files(&mut self, src_path_base: PathBuf) -> Result<(), io::Error> {
        for result in Walk::new(&src_path_base) {
            match result {
                Ok(entry) => {
                    let t = entry.file_type().expect("file type");
                    if t.is_dir() {
                        continue;
                    }
                    let p = entry.path();
                    if p.extension().unwrap() != "md" {
                        continue;
                    }
                    let src_path_rel = entry
                        .path()
                        .strip_prefix(&src_path_base)
                        .expect("is prefix")
                        .to_path_buf();
                    self.docs.push(Doc::new(&src_path_base, src_path_rel));
                }
                Err(err) => error!("Not an entry: {}", err),
            }
        }
        info!("Found {} md files", self.docs.len());
        for d in &mut self.docs {
            d.gen_html()?;
        }
        Ok(())
    }
}

pub fn read_md_files(src_path_base: PathBuf) -> Result<Vec<Doc>, io::Error> {
    let mut conv = HtmlConverter::new();
    conv.read_md_files(src_path_base)?;
    Ok(conv.docs)
}
