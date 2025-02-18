use std::hash::{Hash, Hasher};


use image::{DynamicImage, Rgba };


use image::imageops::FilterType;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Span, Style, Text};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

pub const PIXEL: char = '▀';
const FILTER_TYPE: FilterType = FilterType::Lanczos3;

// use to reduce the size of the image before storing it
const CACHED_WIDTH: u32 = 256;
const CACHED_HEIGHT: u32 = 256;

const GRAY: Rgba<u8> = Rgba([242, 242, 242, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
#[derive(Clone, Debug, PartialEq)]
pub struct CachedImageData {
    image: DynamicImage,
}
impl Hash for CachedImageData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.image.to_rgb8().hash(state);
    }
}


impl CachedImageData {
    pub fn new(rgba_image: DynamicImage) -> Self{
        CachedImageData{image: rgba_image}
    }

    pub fn height(&self) -> u32{
        self.image.height()
    }
    pub fn width(&self) -> u32{
        self.image.width()
    }
    pub fn from_dynamic_image(
        dynamic_image: DynamicImage,
    ) -> Self {
        // if the image is smaller than the preview window, keep it small
        let resized_image = if dynamic_image.width() > CACHED_WIDTH || dynamic_image.height() > CACHED_HEIGHT {
            dynamic_image.resize(CACHED_WIDTH, CACHED_HEIGHT , FilterType::Nearest)
        } else {
            dynamic_image
        };

        //convert the buffer pixels into rgba8
        let rgba_image = resized_image.into_rgba8();
        CachedImageData::new(DynamicImage::from(rgba_image))
    }
    pub fn paragraph<'a>(&self, inner: Rect, preview_block: Block<'a>) -> Paragraph<'a> {
        let preview_width = u32::from(inner.width);
        let preview_height = u32::from(inner.height) * 2; // *2 because 2 pixels per character
        let image_rgba = if self.image.width() > preview_width || self.image.height() > preview_height {
            &self.image.resize(preview_width, preview_height, FILTER_TYPE).into_rgba8()
        } else{
            self.image.as_rgba8().expect("to be rgba8 image") // converted into rgba8 before being put into the cache, so it should never enter the expect
        };
        // transform it into text
        let lines = image_rgba
            // iter over pair of rows
            .rows()
            .step_by(2)
            .zip(image_rgba.rows().skip(1).step_by(2)).enumerate()
            .map(|(double_row_y, (row_1, row_2))|
                Line::from_iter(row_1.into_iter().zip(row_2.into_iter()).enumerate().map(
                    |(x, (color_up, color_down))| {
                        convert_pixel_to_span(
                            color_up,
                            color_down,
                            (x, double_row_y),
                        )}
                )
            ))
            .collect::<Vec<Line>>();
        let text_image = Text::from(lines);
        Paragraph::new(text_image)
            .block(preview_block)
            .alignment(Alignment::Center)
    }

}

#[allow(dead_code)]
fn calculate_fit_dimensions(original_width: u32, original_height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    if original_width <= max_width && original_height <= max_height {
        return (original_width, original_height);
    }

    let width_ratio = f64::from(max_width) / f64::from(original_width);
    let height_ratio = f64::from(max_height) / f64::from(original_height);

    let scale = width_ratio.min(height_ratio);


    let new_width = u32::try_from((f64::from(original_width) * scale).round() as u64)
        .unwrap_or(u32::MAX);
    let new_height =  u32::try_from((f64::from(original_height) * scale).round() as u64)
        .unwrap_or(u32::MAX);

    (new_width, new_height)
}

pub fn convert_pixel_to_span<'a>(
    color_up: &Rgba<u8>,
    color_down: &Rgba<u8>,
    position: (usize, usize),
) -> Span<'a> {
    let color_up = color_up.0;
    let color_down = color_down.0;

    // there is no in between, ether it is transparent, either it use the color
    let alpha_threshold = 30;
    let color_up = if color_up[3] <= alpha_threshold {
        // choose the good color for the background if transparent
        if (position.0 + position.1 * 2) % 2 == 0 {
            WHITE
        } else {
            GRAY
        }
    } else {
        Rgba::from(color_up)
    };
    let color_down = if color_down[3] <= alpha_threshold {
        if (position.0 + position.1 * 2 + 1) % 2 == 0 {
            WHITE
        } else {
            GRAY
        }
    } else {
        Rgba::from(color_down)
    };

    let color_up = convert_image_color_to_ratatui_color(color_up);
    let color_down = convert_image_color_to_ratatui_color(color_down);


    let style = Style::default().fg(color_up).bg(color_down);
    Span::styled(String::from(PIXEL), style)
}

fn convert_image_color_to_ratatui_color(color: Rgba<u8>) -> Color {
    Color::Rgb(color[0], color[1], color[2])
}
