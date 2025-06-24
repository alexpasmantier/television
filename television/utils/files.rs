use crate::utils::{
    strings::{
        PRINTABLE_ASCII_THRESHOLD, proportion_of_printable_ascii_characters,
    },
    threads::default_num_threads,
};
use rustc_hash::FxHashSet;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    sync::OnceLock,
};
use tracing::{debug, warn};

pub struct PartialReadResult {
    pub lines: Vec<String>,
    pub bytes_read: usize,
}

pub enum ReadResult {
    Partial(PartialReadResult),
    Full(Vec<String>),
    Error(String),
}

pub fn read_into_lines_capped<R>(r: R, max_bytes: usize) -> ReadResult
where
    R: Read,
{
    let mut buf_reader = BufReader::new(r);
    let mut line = String::new();
    let mut lines = Vec::new();
    let mut bytes_read = 0;

    loop {
        line.clear();
        match buf_reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if bytes_read > max_bytes {
                    break;
                }
                lines.push(line.trim_end().to_string());
                bytes_read += line.len();
            }
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                return ReadResult::Error(format!("{e:?}"));
            }
        }
    }

    if bytes_read > max_bytes {
        ReadResult::Partial(PartialReadResult { lines, bytes_read })
    } else {
        ReadResult::Full(lines)
    }
}

pub static DEFAULT_NUM_THREADS: OnceLock<usize> = OnceLock::new();

pub fn get_default_num_threads() -> usize {
    *DEFAULT_NUM_THREADS.get_or_init(default_num_threads)
}

pub fn get_file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

#[derive(Debug)]
pub enum FileType {
    Text,
    Image,
    Other,
    Unknown,
}

impl<P> From<P> for FileType
where
    P: AsRef<Path> + Debug,
{
    fn from(path: P) -> Self {
        debug!("Getting file type for {:?}", path);
        let p = path.as_ref();
        if is_accepted_image_extension(p) {
            return FileType::Image;
        }
        if is_known_text_extension(p) {
            return FileType::Text;
        }
        if let Ok(mut f) = File::open(p) {
            let mut buffer = [0u8; 256];
            if let Ok(bytes_read) = f.read(&mut buffer) {
                if bytes_read > 0
                    && proportion_of_printable_ascii_characters(
                        &buffer[..bytes_read],
                    ) > PRINTABLE_ASCII_THRESHOLD
                {
                    return FileType::Text;
                }
            }
        } else {
            warn!("Error opening file: {:?}", path);
        }
        FileType::Other
    }
}

pub fn is_known_text_extension<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| get_known_text_file_extensions().contains(ext))
}

pub static KNOWN_TEXT_FILE_EXTENSIONS: OnceLock<FxHashSet<&'static str>> =
    OnceLock::new();

pub fn get_known_text_file_extensions() -> &'static FxHashSet<&'static str> {
    KNOWN_TEXT_FILE_EXTENSIONS.get_or_init(|| {
        [
            "ada",
            "adb",
            "ads",
            "applescript",
            "as",
            "asc",
            "ascii",
            "ascx",
            "asm",
            "asmx",
            "asp",
            "aspx",
            "atom",
            "au3",
            "awk",
            "bas",
            "bash",
            "bashrc",
            "bat",
            "bbcolors",
            "bcp",
            "bdsgroup",
            "bdsproj",
            "bib",
            "bowerrc",
            "c",
            "cbl",
            "cc",
            "cfc",
            "cfg",
            "cfm",
            "cfml",
            "cgi",
            "cjs",
            "clj",
            "cljs",
            "cls",
            "cmake",
            "cmd",
            "cnf",
            "cob",
            "code-snippets",
            "coffee",
            "coffeekup",
            "conf",
            "cp",
            "cpp",
            "cpt",
            "cpy",
            "crt",
            "cs",
            "csh",
            "cson",
            "csproj",
            "csr",
            "css",
            "csslintrc",
            "csv",
            "ctl",
            "curlrc",
            "cxx",
            "d",
            "dart",
            "dfm",
            "diff",
            "dof",
            "dpk",
            "dpr",
            "dproj",
            "dtd",
            "eco",
            "editorconfig",
            "ejs",
            "el",
            "elm",
            "emacs",
            "eml",
            "ent",
            "erb",
            "erl",
            "eslintignore",
            "eslintrc",
            "ex",
            "exs",
            "f",
            "f03",
            "f77",
            "f90",
            "f95",
            "fish",
            "for",
            "fpp",
            "frm",
            "fs",
            "fsproj",
            "fsx",
            "ftn",
            "gemrc",
            "gemspec",
            "gitattributes",
            "gitconfig",
            "gitignore",
            "gitkeep",
            "gitmodules",
            "go",
            "gpp",
            "gradle",
            "graphql",
            "groovy",
            "groupproj",
            "grunit",
            "gtmpl",
            "gvimrc",
            "h",
            "haml",
            "hbs",
            "hgignore",
            "hh",
            "hpp",
            "hrl",
            "hs",
            "hta",
            "htaccess",
            "htc",
            "htm",
            "html",
            "htpasswd",
            "hxx",
            "iced",
            "iml",
            "inc",
            "inf",
            "info",
            "ini",
            "ino",
            "int",
            "irbrc",
            "itcl",
            "itermcolors",
            "itk",
            "jade",
            "java",
            "jhtm",
            "jhtml",
            "js",
            "jscsrc",
            "jshintignore",
            "jshintrc",
            "json",
            "json5",
            "jsonld",
            "jsp",
            "jspx",
            "jsx",
            "ksh",
            "less",
            "lhs",
            "lisp",
            "log",
            "ls",
            "lsp",
            "lua",
            "m",
            "m4",
            "mak",
            "map",
            "markdown",
            "master",
            "md",
            "mdown",
            "mdwn",
            "mdx",
            "metadata",
            "mht",
            "mhtml",
            "mjs",
            "mk",
            "mkd",
            "mkdn",
            "mkdown",
            "ml",
            "mli",
            "mm",
            "mxml",
            "nfm",
            "nfo",
            "noon",
            "npmignore",
            "npmrc",
            "nuspec",
            "nvmrc",
            "ops",
            "pas",
            "pasm",
            "patch",
            "pbxproj",
            "pch",
            "pem",
            "pg",
            "php",
            "php3",
            "php4",
            "php5",
            "phpt",
            "phtml",
            "pir",
            "pl",
            "pm",
            "pmc",
            "pod",
            "pot",
            "prettierrc",
            "properties",
            "props",
            "pt",
            "pug",
            "purs",
            "py",
            "pyx",
            "r",
            "rake",
            "rb",
            "rbw",
            "rc",
            "rdoc",
            "rdoc_options",
            "resx",
            "rexx",
            "rhtml",
            "rjs",
            "rlib",
            "ron",
            "rs",
            "rss",
            "rst",
            "rtf",
            "rvmrc",
            "rxml",
            "s",
            "sass",
            "scala",
            "scm",
            "scss",
            "seestyle",
            "sh",
            "shtml",
            "sln",
            "sls",
            "spec",
            "sql",
            "sqlite",
            "sqlproj",
            "srt",
            "ss",
            "sss",
            "st",
            "strings",
            "sty",
            "styl",
            "stylus",
            "sub",
            "sublime-build",
            "sublime-commands",
            "sublime-completions",
            "sublime-keymap",
            "sublime-macro",
            "sublime-menu",
            "sublime-project",
            "sublime-settings",
            "sublime-workspace",
            "sv",
            "svc",
            "svg",
            "swift",
            "t",
            "tcl",
            "tcsh",
            "terminal",
            "tex",
            "text",
            "textile",
            "tg",
            "tk",
            "tmLanguage",
            "tmpl",
            "tmTheme",
            "toml",
            "tpl",
            "ts",
            "tsv",
            "tsx",
            "tt",
            "tt2",
            "ttml",
            "twig",
            "txt",
            "v",
            "vb",
            "vbproj",
            "vbs",
            "vcproj",
            "vcxproj",
            "vh",
            "vhd",
            "vhdl",
            "vim",
            "viminfo",
            "vimrc",
            "vm",
            "vue",
            "webapp",
            "webmanifest",
            "wsc",
            "x-php",
            "xaml",
            "xht",
            "xhtml",
            "xml",
            "xs",
            "xsd",
            "xsl",
            "xslt",
            "y",
            "yaml",
            "yml",
            "zsh",
            "zshrc",
        ]
        .iter()
        .copied()
        .collect()
    })
}

pub fn is_accepted_image_extension<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| get_known_image_file_extensions().contains(ext))
}
pub static KNOWN_IMAGE_FILE_EXTENSIONS: OnceLock<FxHashSet<&'static str>> =
    OnceLock::new();
pub fn get_known_image_file_extensions() -> &'static FxHashSet<&'static str> {
    KNOWN_IMAGE_FILE_EXTENSIONS.get_or_init(|| {
        [
            // "avif", requires the avif-native feature, uses the libdav1d C library.
            // dds, dosen't work for some reason
            "bmp", "ff", "gif", "hdr", "ico", "jpeg", "jpg", "exr", "png",
            "pnm", "qoi", "tga", "tif", "webp",
        ]
        .iter()
        .copied()
        .collect()
    })
}
