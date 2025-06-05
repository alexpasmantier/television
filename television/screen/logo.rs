use ratatui::widgets::Paragraph;

//const LOGO: &str = r"                                           _______________
//    __      __         _     _            |,----------.  |\
//   / /____ / /__ _  __(_)__ (_)__  ___    ||           |=| |
//  / __/ -_) / -_) |/ / (_-</ / _ \/ _ \   ||           | | |
//  \__/\__/_/\__/|___/_/___/_/\___/_//_/   ||           |o| |
//                                          |`-----------' |/
//                                          `--------------'";

const LOGO: &str = r"  _______________
 |,----------.  |\
 ||           |=| |
 ||           | | |
 ||           |o| |
 |`-----------' |/ 
 `--------------'";

pub fn build_logo_paragraph<'a>() -> Paragraph<'a> {
    let lines = LOGO
        .lines()
        .map(std::convert::Into::into)
        .collect::<Vec<_>>();
    Paragraph::new(lines)
}

const REMOTE_LOGO: &str = r"
 _____________
/             \
| (*)     (#) |
|             |
| (1) (2) (3) |
| (4) (5) (6) |
| (7) (8) (9) |
|             |
|      _      |
|     | |     |
|  (_¯(0)¯_)  |
|     | |     |
|      ¯      |
|             |
|             |
| === === === |
|             |
|     T.V     |
`-------------´";

pub fn build_remote_logo_paragraph<'a>() -> Paragraph<'a> {
    let lines = REMOTE_LOGO
        .lines()
        .map(std::convert::Into::into)
        .collect::<Vec<_>>();
    Paragraph::new(lines)
}
