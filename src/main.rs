#![feature(path_file_prefix)]

use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use sarge::{arg, get_flag, get_val, ArgValue, ArgumentParser};
use xml::reader::{EventReader, XmlEvent};

fn file_to_url(mut file: &Path, root: String, oldroot: Option<String>, clean: bool) -> String {
    if oldroot.is_some() {
        file = if let Ok(f) = file.strip_prefix(&oldroot.unwrap()) {
            f
        } else {
            file
        };
    }

    let mut path = PathBuf::new();
    path.push(&root);

    println!("{:?}", file.file_prefix());
    if clean && file.file_prefix().is_some() && file.file_prefix().unwrap() == "index" {
        path.push(file.ancestors().skip(1).collect::<PathBuf>().as_path());
    } else {
        path.push(file);
    }

    path.to_string_lossy().to_string()
}

fn escape(l: &String) -> String {
    l.replace('&', "&amp;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
        .replace(">", "&gt;")
        .replace("<", "&lt;")
}

#[derive(Debug, Clone, PartialEq)]
struct Url {
    pub loc: String,

    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<String>,
}

impl Url {
    pub fn new(loc: String) -> Self {
        Self {
            loc,
            lastmod: None,
            changefreq: None,
            priority: None,
        }
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    <url>")?;
        writeln!(f, "        <loc>{}</loc>", escape(&self.loc))?;

        if let Some(lastmod) = &self.lastmod {
            writeln!(f, "        <lastmod>{}</lastmod>", escape(lastmod))?;
        }

        if let Some(changefreq) = &self.changefreq {
            writeln!(f, "        <changefreq>{}</changefreq>", escape(changefreq))?;
        }

        if let Some(priority) = &self.priority {
            writeln!(f, "        <priority>{}</priority>", escape(priority))?;
        }

        writeln!(f, "    </url>")
    }
}

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(flag, both, 'c', "clean"));
    parser.add(arg!(str, both, 'f', "file"));

    parser.add(arg!(str, both, 'a', "add"));
    parser.add(arg!(str, long, "lastmod"));
    parser.add(arg!(str, long, "changefreq"));
    parser.add(arg!(str, long, "priority"));

    parser.add(arg!(str, both, 'r', "remove"));

    parser.add(arg!(flag, long, "to-url"));
    parser.add(arg!(str, long, "root"));
    parser.add(arg!(str, long, "old-root"));
    parser.add(arg!(flag, long, "clean-url"));

    parser.parse().expect("Failed to parse arguments");

    if get_flag!(parser, both, 'h', "help") {
        println!("{} [options]", parser.binary.unwrap_or_else(|| String::from("onsite")));
        println!("        -h |  --help : print this help message");
        println!("        -c | --clean : remove all urls from the sitemap");
        println!("        -f |  --file : specify the sitemap file (default: sitemap.xml)");
        println!();
        println!("    -a | --add <url> : add a url to the sitemap (gets escaped)");
        println!("    --lastmod <date> : set the url's lastmod property");
        println!(" --changefreq <freq> : set the url's changefreq property");
        println!("    --priority <pri> : set the url's priority property");
        println!();
        println!(" -r | --remove <url> : remove a url from the sitemap");
        println!();
        println!("            --to-url : transform a filepath into a url (requires --root)");
        println!("       --root <root> : the root (plus protocol) for the url");
        println!("    --old-root <old> : a prefix to strip from the filepath");
        println!("         --clean-url : removes `index.*` from the end of the filepath");
        return;
    }

    let path = if let Some(ArgValue::String(f)) = get_val!(parser, both, 'f', "file") {
        f
    } else {
        String::from("sitemap.xml")
    };

    let mut url = None;
    let mut urls = Vec::new();
    let mut names = Vec::new();

    if !get_flag!(parser, both, 'c', "clean") {
        let file = File::open(&path).expect("Failed to open file for reading");
        let file = BufReader::new(file);

        let parser = EventReader::new(file);
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    // println!("{}<{}>", indent(depth), name.local_name);
                    // depth += 1;
                    names.push(name);
                }
                Ok(XmlEvent::EndElement { name }) => {
                    // depth -= 1;
                    // println!("{}</{}>", indent(depth), name.local_name);
                    if name.local_name == "url" {
                        urls.push(url.take().unwrap());
                    }

                    names.pop();
                }
                Ok(XmlEvent::Characters(data)) => {
                    // println!("{}{}", indent(depth), data);

                    if let Some(name) = names.last() {
                        match name.local_name.as_str() {
                            "loc" => url = Some(Url::new(data)),
                            "lastmod" => url.as_mut().unwrap().lastmod = Some(data),
                            "changefreq" => url.as_mut().unwrap().changefreq = Some(data),
                            "priority" => url.as_mut().unwrap().priority = Some(data),
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    if let Some(ArgValue::String(loc)) = get_val!(parser, both, 'r', "remove") {
        let i = urls.iter()
            .enumerate()
            .find(|(_, url)| url.loc == loc)
            .map(|(i, _)| i);

        if let Some(i) = i {
            urls.remove(i);
        }
    }

    if let Some(ArgValue::String(loc)) = get_val!(parser, both, 'a', "add") {
        let mut url = if get_flag!(parser, long, "to-url") {
            Url::new(
                file_to_url(
                    Path::new(&loc),
                    get_val!(parser, long, "root").expect("Must pass `--root` to `--to-url`").get_str(),
                    get_val!(parser, long, "old-root").map(|s| s.get_str()),
                    get_flag!(parser, long, "clean-url")
                )
            )
        } else {
            Url::new(loc)
        };

        if let Some(ArgValue::String(lastmod)) = get_val!(parser, long, "lastmod") {
            url.lastmod = Some(lastmod);
        }

        if let Some(ArgValue::String(changefreq)) = get_val!(parser, long, "changefreq") {
            url.changefreq = Some(changefreq);
        }

        if let Some(ArgValue::String(priority)) = get_val!(parser, long, "priority") {
            url.priority = Some(priority);
        }

        urls.push(url);
    }

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Failed to open file for writing");

    let mut file = BufWriter::new(file);

    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").expect("Failed to write to file");
    writeln!(file, "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">").expect("Failed to write to file");

    for url in urls {
        writeln!(file, "{url}").expect("Failed to write to file");
    }

    writeln!(file, "</urlset>").expect("Failed to write to file");
}