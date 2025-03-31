use criterion::{criterion_group, Criterion};
use devicons::FileIcon;
use ratatui::layout::Alignment;
use ratatui::prelude::{Line, Style};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, ListDirection, Padding};
use television::channels::entry::into_ranges;
use television::channels::entry::{Entry, PreviewType};
use television::screen::colors::ResultsColorscheme;
use television::screen::results::build_results_list;

pub fn draw_results_list(c: &mut Criterion) {
    // FIXME: there's  probably a way to have this as a benchmark asset
    // possible as a JSON file and to load it for the benchmark using Serde
    // I don't know how exactly right now just having it here instead
    let entries = [
        Entry {
            name: "typeshed/LICENSE".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{f016}',
                color: "#7e8e91",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/README.md".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{f48a}',
                color: "#dddddd",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/re.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/io.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/gc.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/uu.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/nt.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/dis.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/imp.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/bdb.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/abc.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/cgi.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/bz2.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/grp.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/ast.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/csv.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/pdb.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/pwd.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/ssl.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/tty.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/nis.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/pty.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/cmd.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/tests/utils.py".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/pyproject.toml".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e6b2}',
                color: "#9c4221",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/MAINTAINERS.md".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{f48a}',
                color: "#dddddd",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/enum.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/hmac.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/uuid.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/glob.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/_ast.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/_csv.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/code.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/spwd.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/_msi.pyi".to_string(),
            value: None,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
        },
        Entry {
            name: "typeshed/stdlib/time.pyi".to_string(),
            value: None,

            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            preview_type: PreviewType::Files,
            name_match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            value_match_ranges: None,
        },
    ];

    let colorscheme = ResultsColorscheme {
        result_name_fg: Color::Indexed(222),
        result_preview_fg: Color::Indexed(222),
        result_line_number_fg: Color::Indexed(222),
        result_selected_fg: Color::Indexed(222),
        result_selected_bg: Color::Indexed(222),
        match_foreground_color: Color::Indexed(222),
    };

    c.bench_function("results_list", |b| {
        b.iter(|| {
            build_results_list(
                Block::default()
                    .title_top(
                        Line::from(" Results ").alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .style(Style::default())
                    .padding(Padding::right(1)),
                &entries,
                None,
                ListDirection::BottomToTop,
                false,
                &colorscheme,
                100,
            );
        });
    });
}

criterion_group!(benches, draw_results_list);
