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
    let logo_paragraph = Paragraph::new(lines);
    logo_paragraph
}
