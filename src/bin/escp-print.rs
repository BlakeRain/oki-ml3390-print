use clap::Parser;
use humansize::{format_size, BINARY};
use std::{io::BufRead, path::Path};
use synoptic::{languages, Highlighter, Token};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

#[derive(Debug, Parser)]
struct Options {
    #[clap(short, long)]
    header: bool,
    #[clap(short, long, value_parser, value_name = "TITLE")]
    title: Option<String>,
    #[clap(short, long, value_parser, value_name = "EXTENSION")]
    extension: Option<String>,
    paths: Vec<std::path::PathBuf>,
}

const CODE_BOLD_ENABLE: &str = "\x1bE";
const CODE_BOLD_DISABLE: &str = "\x1bF";
const CODE_ITALIC_ENABLE: &str = "\x1b4";
const CODE_ITALIC_DISABLE: &str = "\x1b5";
const CODE_UNDERLINE_ENABLE: &str = "\x1b-1";
const CODE_UNDERLINE_DISABLE: &str = "\x1b-0";

struct HeaderStat {
    title: String,
    value: String,
}

impl HeaderStat {
    fn new(title: String, value: String) -> Self {
        Self { title, value }
    }

    fn from_stat(metadata: std::fs::Metadata) -> Vec<Self> {
        let mut stats = Vec::new();

        if let Some(created) = metadata
            .created()
            .ok()
            .map(|time| time.into())
            .and_then(|time: OffsetDateTime| time.format(&Rfc2822).ok())
        {
            stats.push(Self::new("Created".to_string(), created));
        }

        if let Some(modified) = metadata
            .modified()
            .ok()
            .map(|time| time.into())
            .and_then(|time: OffsetDateTime| time.format(&Rfc2822).ok())
        {
            stats.push(Self::new("Modified".to_string(), modified));
        }

        stats.push(Self::new(
            "File Size".to_string(),
            format_size(metadata.len(), BINARY),
        ));

        stats
    }

    fn from_file(path: &Path) -> Vec<Self> {
        path.metadata().map(Self::from_stat).unwrap_or_default()
    }
}

struct Header {
    title: String,
    stats: Vec<HeaderStat>,
}

impl Header {
    fn for_file(path: &Path, title: &Option<String>) -> Self {
        Self {
            title: title
                .clone()
                .or_else(|| path.to_str().map(ToOwned::to_owned))
                .unwrap_or_default(),
            stats: HeaderStat::from_file(path),
        }
    }

    fn print(self) {
        let value_max = self
            .stats
            .iter()
            .map(|stat| stat.value.len())
            .max()
            .unwrap_or_default();

        println!(
            "\x1bw1{}{}{}\x1bw0",
            CODE_BOLD_ENABLE, self.title, CODE_BOLD_DISABLE
        );

        print!("\x1ba2");
        for stat in self.stats.into_iter() {
            println!(
                "{}{}: {:width$}{}",
                CODE_BOLD_ENABLE,
                stat.title,
                stat.value,
                CODE_BOLD_DISABLE,
                width = value_max
            );
        }

        println!("\x1ba0\n");
    }
}

fn print_string(highlighter: &Option<Highlighter>, content: String) {
    if let Some(highlighter) = highlighter {
        for (c, row) in highlighter.run(&content).iter().enumerate() {
            print!("{:0>5}", 1 + c);

            for token in row {
                match token {
                    Token::Text(txt) => print!("{}", txt),
                    Token::Start(kind) => match kind.as_str() {
                        "keyword" => print!("{}", CODE_BOLD_ENABLE),
                        "comment" => print!("{}", CODE_ITALIC_ENABLE),
                        "string" => print!("{}", CODE_UNDERLINE_ENABLE),
                        _ => (),
                    },
                    Token::End(kind) => match kind.as_str() {
                        "keyword" => print!("{}", CODE_BOLD_DISABLE),
                        "comment" => print!("{}", CODE_ITALIC_DISABLE),
                        "string" => print!("{}", CODE_UNDERLINE_DISABLE),
                        _ => (),
                    },
                }
            }

            println!();
        }
    } else {
        print!("{}", content);
    }
}

fn print_file(highlighter: &Option<Highlighter>, path: std::path::PathBuf) {
    let content = std::fs::read_to_string(path).unwrap();
    print_string(highlighter, content);
}

fn print_stdin(highlighter: &Option<Highlighter>) {
    let mut content = String::new();
    for buffer in std::io::stdin().lock().lines() {
        match buffer {
            Err(err) => panic!("Unable to read from stdin: {:?}", err),
            Ok(buffer) => {
                content.push_str(&buffer);
            }
        }
    }

    print_string(highlighter, content);
}

fn find_highlighter_for_extension(ext: &str) -> Option<Highlighter> {
    match ext {
        "rs" => Some(languages::rust()),
        "py" => Some(languages::python()),
        _ => None,
    }
}

fn find_highlighter_for_path(ext: Option<String>, path: &Path) -> Option<Highlighter> {
    if let Some(path_ext) = path.extension() {
        if let Some(path_ext) = path_ext.to_str() {
            return find_highlighter_for_extension(path_ext);
        }
    }

    if let Some(ext) = ext {
        find_highlighter_for_extension(&ext)
    } else {
        None
    }
}

fn main() {
    let options = Options::parse();

    if !options.paths.is_empty() {
        for path in options.paths {
            if options.header {
                let header = Header::for_file(&path, &options.title);
                header.print();
            }

            let highlighter = find_highlighter_for_path(options.extension.clone(), &path);
            print_file(&highlighter, path);
        }
    } else {
        let highlighter = options
            .extension
            .and_then(|ext| find_highlighter_for_extension(&ext));
        print_stdin(&highlighter);
    }
}
