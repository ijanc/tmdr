// vim: set tw=79 cc=80 ts=4 sw=4 sts=4 et :
//
// Copyright (c) 2026 Murilo Ijanc' <murilo@ijanc.org>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use std::{
    env, fs,
    io::{self, Read, Write},
};

use markdown::{ParseOptions, mdast::Node, to_mdast};

fn main() {
    let args: Vec<String> = env::args().collect();
    let (cols, file) = parse_args(&args);

    let input = read_input(file);
    let tree = match to_mdast(&input, &ParseOptions::default()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tmdr: {}", e);
            std::process::exit(1);
        }
    };

    let out = io::stdout();
    let mut w = out.lock();
    render(&mut w, &tree, cols);
}

fn parse_args(args: &[String]) -> (usize, Option<&str>) {
    let mut cols = 80usize;
    let mut file = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-V" => {
                println!("tmdr {}", env!("TMDR_VERSION"));
                std::process::exit(0);
            }
            "-w" => {
                i += 1;
                cols = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(80);
            }
            s if !s.starts_with('-') => file = Some(s),
            _ => usage(),
        }
        i += 1;
    }
    (cols, file)
}

fn usage() -> ! {
    eprintln!("usage: tmdr [-V] [-w cols] [file]");
    std::process::exit(1)
}

fn read_input(file: Option<&str>) -> String {
    match file {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("tmdr: {}: {}", path, e);
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("tmdr: {}", e);
                std::process::exit(1);
            });
            buf
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// Render
//////////////////////////////////////////////////////////////////////////////

const BOLD: &str = "\x1b[1m";
const BOLD_OFF: &str = "\x1b[22m";
const ITALIC: &str = "\x1b[3m";
const ITALIC_OFF: &str = "\x1b[23m";
const UL: &str = "\x1b[4m";
const RESET: &str = "\x1b[0m";

fn render(w: &mut impl Write, node: &Node, cols: usize) {
    let children = match node {
        Node::Root(r) => &r.children,
        _ => return,
    };

    let mut first = true;
    for child in children {
        if !first {
            let _ = writeln!(w);
        }
        first = false;
        render_block(w, child, cols);
    }
}

fn render_block(w: &mut impl Write, node: &Node, cols: usize) {
    match node {
        Node::Heading(h) => {
            let prefix = "#".repeat(h.depth as usize);
            let text = inline_text(&h.children);
            let _ = writeln!(w, "{}{}{} {}{}", BOLD, UL, prefix, text, RESET);
        }
        Node::Paragraph(p) => {
            let text = inline_text(&p.children);
            print_wrapped(w, &text, cols, "");
        }
        Node::Code(c) => {
            for line in c.value.lines() {
                let _ = writeln!(w, "    {}", line);
            }
        }
        Node::List(list) => {
            for (i, child) in list.children.iter().enumerate() {
                if let Node::ListItem(item) = child {
                    let marker = if list.ordered {
                        let n = list.start.unwrap_or(1) as usize + i;
                        format!("{}. ", n)
                    } else {
                        "- ".to_string()
                    };
                    let text = list_item_text(&item.children);
                    print_wrapped(w, &text, cols, &marker);
                }
            }
        }
        Node::ThematicBreak(_) => {
            let _ = writeln!(w, "{}", "\u{2500}".repeat(cols.min(72)));
        }
        Node::Blockquote(bq) => {
            for child in &bq.children {
                render_block(w, child, cols.saturating_sub(2));
            }
        }
        _ => {}
    }
}

fn inline_text(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            Node::Text(t) => out.push_str(&t.value),
            Node::Strong(s) => {
                out.push_str(BOLD);
                out.push_str(&inline_text(&s.children));
                out.push_str(BOLD_OFF);
            }
            Node::Emphasis(e) => {
                out.push_str(ITALIC);
                out.push_str(&inline_text(&e.children));
                out.push_str(ITALIC_OFF);
            }
            Node::InlineCode(c) => out.push_str(&c.value),
            Node::Break(_) => out.push('\n'),
            Node::Link(l) => {
                let text = inline_text(&l.children);
                out.push_str("\x1b]8;;");
                out.push_str(&l.url);
                out.push_str("\x1b\\");
                out.push_str(UL);
                out.push_str(&text);
                out.push_str(RESET);
                out.push_str("\x1b]8;;\x1b\\");
            }
            Node::Image(img) => out.push_str(&img.alt),
            _ => {}
        }
    }
    out
}

fn list_item_text(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        if let Node::Paragraph(p) = node {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&inline_text(&p.children));
        }
    }
    out
}

fn print_wrapped(w: &mut impl Write, text: &str, cols: usize, prefix: &str) {
    let plen = visible_len(prefix);
    let wrap_at = cols.saturating_sub(plen);
    let indent: String = " ".repeat(plen);
    let mut first_line = true;

    for segment in text.split('\n') {
        let words: Vec<&str> = segment.split_whitespace().collect();
        if words.is_empty() {
            let _ = writeln!(w);
            first_line = false;
            continue;
        }

        let pfx = if first_line { prefix } else { &indent };
        first_line = false;
        let _ = write!(w, "{}", pfx);
        let mut col = 0usize;

        for word in &words {
            let wlen = visible_len(word);
            if col > 0 && col + 1 + wlen > wrap_at {
                let _ = writeln!(w);
                let _ = write!(w, "{}", indent);
                col = 0;
            }
            if col > 0 {
                let _ = write!(w, " ");
                col += 1;
            }
            let _ = write!(w, "{}", word);
            col += wlen;
        }
        let _ = writeln!(w);
    }
}

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_esc = false;
    for c in s.chars() {
        if in_esc {
            if c == 'm' {
                in_esc = false;
            }
        } else if c == '\x1b' {
            in_esc = true;
        } else {
            len += 1;
        }
    }
    len
}
