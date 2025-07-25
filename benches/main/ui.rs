use criterion::criterion_group;
use criterion::{Criterion, black_box};
use devicons::FileIcon;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Style};
use ratatui::style::Color;
use ratatui::widgets::{
    Block, BorderType, Borders, ListDirection, ListState, Padding,
};
use television::channels::prototypes::ChannelPrototype;
use television::picker::Movement;
use television::{
    action::Action,
    cable::Cable,
    channels::entry::{Entry, into_ranges},
    cli::PostProcessedCli,
    config::{Config, ConfigEnv},
    screen::{colors::ResultsColorscheme, result_item::build_results_list},
    television::Television,
};
use tokio::runtime::Runtime;

#[allow(clippy::too_many_lines)]
pub fn draw_results_list(c: &mut Criterion) {
    // FIXME: there's  probably a way to have this as a benchmark asset
    // possible as a JSON file and to load it for the benchmark using Serde
    // I don't know how exactly right now just having it here instead
    let entries = [
        Entry {
            display: None,
            raw: "typeshed/LICENSE".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{f016}',
                color: "#7e8e91",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/README.md".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{f48a}',
                color: "#dddddd",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/re.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/io.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/gc.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/uu.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/nt.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/dis.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/imp.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/bdb.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/abc.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/cgi.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/bz2.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/grp.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/ast.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/csv.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/pdb.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/pwd.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/ssl.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/tty.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/nis.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/pty.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/cmd.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/tests/utils.py".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/pyproject.toml".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e6b2}',
                color: "#9c4221",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/MAINTAINERS.md".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{f48a}',
                color: "#dddddd",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/enum.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/hmac.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/uuid.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/glob.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/_ast.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/_csv.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/code.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/spwd.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/_msi.pyi".to_string(),
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            ansi: false,
        },
        Entry {
            display: None,
            raw: "typeshed/stdlib/time.pyi".to_string(),

            icon: Some(FileIcon {
                icon: '\u{e606}',
                color: "#ffbc03",
            }),
            line_number: None,
            match_ranges: Some(into_ranges(&[0, 1, 2, 3])),
            ansi: false,
        },
    ];

    let colorscheme = ResultsColorscheme {
        result_fg: Color::Indexed(222),
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
                &ListState::default(),
                ListDirection::BottomToTop,
                false,
                &colorscheme,
                100,
                |_| None,
            );
        });
    });
}

#[allow(clippy::missing_panics_doc)]
pub fn draw(c: &mut Criterion) {
    let width = 250;
    let height = 80;

    let rt = Runtime::new().unwrap();

    let cable = Cable::from_prototypes(vec![ChannelPrototype::new(
        "files", "fd -t f",
    )]);

    c.bench_function("draw", |b| {
        b.to_async(&rt).iter_batched(
            // FIXME: this is kind of hacky
            || {
                let config =
                    Config::new(&ConfigEnv::init().unwrap(), None).unwrap();
                let backend = TestBackend::new(width, height);
                let terminal = Terminal::new(backend).unwrap();
                let (tx, _) = tokio::sync::mpsc::unbounded_channel();
                let channel_prototype = cable.get_channel("files");
                // Wait for the channel to finish loading
                let mut tv = Television::new(
                    tx,
                    channel_prototype,
                    config,
                    cable.clone(),
                    PostProcessedCli::default(),
                );
                tv.find("television");
                for _ in 0..5 {
                    // tick the matcher
                    let _ = tv.channel.results(10, 0);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                tv.move_cursor(Movement::Next, 10);
                let selected_entry = tv.get_selected_entry();
                let _ = tv.update_preview_state(&selected_entry);
                let _ = tv.update(&Action::Tick);
                (tv, terminal)
            },
            // Measurement
            |(tv, mut terminal)| async move {
                television::draw::draw(
                    black_box(Box::new(tv.dump_context())),
                    black_box(&mut terminal.get_frame()),
                    black_box(Rect::new(0, 0, width, height)),
                )
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, draw_results_list, draw);
