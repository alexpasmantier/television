/*
MIT License

Copyright (c) 2023 - sxyazi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::ffi::OsString;

use crate::utils::rocell::RoCell;
use parking_lot::Mutex;

pub static CLIPBOARD: RoCell<Clipboard> = RoCell::new();

#[derive(Default)]
pub struct Clipboard {
    content: Mutex<OsString>,
}

impl Clipboard {
    #[cfg(unix)]
    pub async fn get(&self) -> OsString {
        use std::os::unix::prelude::OsStringExt;

        use tokio::process::Command;

        let all = [
            ("pbpaste", &[][..]),
            ("termux-clipboard-get", &[]),
            ("wl-paste", &[]),
            ("xclip", &["-o", "-selection", "clipboard"]),
            ("xsel", &["-ob"]),
        ];

        for (bin, args) in all {
            let Ok(output) = Command::new(bin)
                .args(args)
                .kill_on_drop(true)
                .output()
                .await
            else {
                continue;
            };
            if output.status.success() {
                return OsString::from_vec(output.stdout);
            }
        }
        self.content.lock().clone()
    }

    #[cfg(windows)]
    pub async fn get(&self) -> OsString {
        use clipboard_win::{formats, get_clipboard};

        let result = tokio::task::spawn_blocking(|| {
            get_clipboard::<String, _>(formats::Unicode)
        });
        if let Ok(Ok(s)) = result.await {
            return s.into();
        }

        self.content.lock().clone()
    }

    #[cfg(unix)]
    pub async fn set(&self, s: impl AsRef<std::ffi::OsStr>) {
        use std::{
            io::{BufWriter, stderr},
            process::Stdio,
        };

        use crossterm::execute;
        use tokio::{io::AsyncWriteExt, process::Command};

        s.as_ref().clone_into(&mut self.content.lock());
        execute!(
            BufWriter::new(stderr()),
            osc52::SetClipboard::new(s.as_ref())
        )
        .ok();

        let all = [
            ("pbcopy", &[][..]),
            ("termux-clipboard-set", &[]),
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["-ib"]),
        ];

        for (bin, args) in all {
            let cmd = Command::new(bin)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn();

            let Ok(mut child) = cmd else { continue };

            let mut stdin = child.stdin.take().unwrap();
            if stdin
                .write_all(s.as_ref().as_encoded_bytes())
                .await
                .is_err()
            {
                continue;
            }
            drop(stdin);

            if child.wait().await.is_ok_and(|s| s.success()) {
                break;
            }
        }
    }

    #[cfg(windows)]
    pub async fn set(&self, s: impl AsRef<std::ffi::OsStr>) {
        use clipboard_win::{formats, set_clipboard};

        let s = s.as_ref().to_owned();
        *self.content.lock() = s.clone();

        tokio::task::spawn_blocking(move || {
            set_clipboard(formats::Unicode, s.to_string_lossy())
        })
        .await
        .ok();
    }
}

#[cfg(unix)]
mod osc52 {
    use std::ffi::OsStr;

    use base64::{Engine, engine::general_purpose};

    #[derive(Debug)]
    pub struct SetClipboard {
        content: String,
    }

    impl SetClipboard {
        pub fn new(content: &OsStr) -> Self {
            Self {
                content: general_purpose::STANDARD
                    .encode(content.as_encoded_bytes()),
            }
        }
    }

    impl crossterm::Command for SetClipboard {
        fn write_ansi(
            &self,
            f: &mut impl std::fmt::Write,
        ) -> std::fmt::Result {
            write!(f, "\x1b]52;c;{}\x1b\\", self.content)
        }
    }
}
