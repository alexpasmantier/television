use std::hash::{Hash, Hasher};

use image::{DynamicImage, Rgba};

use image::imageops::FilterType;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Span, Style, Text};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

const PIXEL: char = '▀';
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
        self.image.as_rgb8().expect("to be rgba image").hash(state);
    }
}

impl CachedImageData {
    pub fn new(image: DynamicImage) -> Self {
        //convert the buffer pixels into rgba8
        let rgba_image = image.into_rgba8();
        CachedImageData {
            image: DynamicImage::from(rgba_image),
        }
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn from_dynamic_image(dynamic_image: DynamicImage) -> Self {
        // if the image is smaller than the preview window, keep it small
        let resized_image = if dynamic_image.width() > CACHED_WIDTH
            || dynamic_image.height() > CACHED_HEIGHT
        {
            dynamic_image.resize(
                CACHED_WIDTH,
                CACHED_HEIGHT,
                FilterType::Nearest,
            )
        } else {
            dynamic_image
        };
        CachedImageData::new(resized_image)
    }
    pub fn paragraph<'a>(
        &self,
        inner: Rect,
        preview_block: Block<'a>,
    ) -> Paragraph<'a> {
        let preview_width = u32::from(inner.width);
        let preview_height = u32::from(inner.height) * 2; // *2 because 2 pixels per character
        let image_rgba = if self.image.width() > preview_width
            || self.image.height() > preview_height
        {
            &self
                .image
                .resize(preview_width, preview_height, FILTER_TYPE)
                .into_rgba8()
        } else {
            self.image.as_rgba8().expect("to be rgba image") // converted into rgba8 before being put into the cache, so it should never enter the expect
        };
        // transform it into text
        let lines = image_rgba
            // iter over pair of rows
            .rows()
            .step_by(2)
            .zip(image_rgba.rows().skip(1).step_by(2))
            .enumerate()
            .map(|(double_row_y, (row_1, row_2))| {
                Line::from_iter(row_1.into_iter().zip(row_2).enumerate().map(
                    |(x, (color_up, color_down))| {
                        convert_pixel_to_span(
                            color_up,
                            color_down,
                            (x, double_row_y),
                        )
                    },
                ))
            })
            .collect::<Vec<Line>>();
        let text_image = Text::from(lines);
        Paragraph::new(text_image)
            .block(preview_block)
            .alignment(Alignment::Center)
    }
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
