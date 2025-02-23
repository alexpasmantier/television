use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Pixel, Rgba, RgbaImage};
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Span, Style, Text};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

const PIXEL: char = '▀';
const FILTER_TYPE: FilterType = FilterType::Lanczos3;

// use to reduce the size of the image before storing it
const CACHED_WIDTH: u32 = 128;
const CACHED_HEIGHT: u32 = 128;

const GRAY: Rgba<u8> = Rgba([242, 242, 242, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

struct Cache {
    area: Rect,
    image: RgbaImage,
}
#[derive(Clone)]
pub struct CachedImageData {
    image: DynamicImage,
    inner_cache: Arc<Mutex<Option<Cache>>>,
}
impl Hash for CachedImageData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.image.as_rgb8().expect("to be rgba image").hash(state);
    }
}
impl Debug for CachedImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedImageData")
            .field("dimensions", &self.image.dimensions()) // Show dimensions instead of full image
            .field("inner_cache", &self.inner_cache.lock().unwrap().is_some()) // Indicate if cache exists
            .finish()
    }
}

impl PartialEq for CachedImageData {
    fn eq(&self, other: &Self) -> bool {
        self.image.eq(&other.image)
    }
}

impl CachedImageData {
    pub fn new(image: DynamicImage) -> Self {
        //convert the buffer pixels into rgba8
        let rgba_image = image.into_rgba8();
        CachedImageData {
            image: DynamicImage::from(rgba_image),
            inner_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    fn cache(&self) -> &Arc<Mutex<Option<Cache>>> {
        &self.inner_cache
    }
    pub fn set_cache(&self, area: Rect, image: RgbaImage) {
        let mut mutex_cache = self.cache().lock().unwrap();
        if let Some(cache) = mutex_cache.as_mut() {
            cache.area = area;
            cache.image = image;
        } else {
            *mutex_cache = Some(Cache { area, image });
        };
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
    fn text_from_rgba_image_ref(image_rgba: &RgbaImage) -> Text<'static> {
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

        Text::from(lines).centered()
    }
    pub fn paragraph<'a>(
        &self,
        inner: Rect,
        preview_block: Block<'a>,
    ) -> Paragraph<'a> {
        let preview_width = u32::from(inner.width);
        let preview_height = u32::from(inner.height) * 2; // *2 because 2 pixels per character
        let text_image = if self.cache().lock().unwrap().is_none()
            || self.cache().lock().unwrap().as_ref().unwrap().area != inner
        {
            let image_rgba = if self.image.width() > preview_width
                || self.image.height() > preview_height
            {
                //warn!("===========================");
                self.image
                    .resize(preview_width, preview_height, FILTER_TYPE)
                    .into_rgba8()
            } else {
                self.image.to_rgba8()
            };

            // transform it into text
            let text = Self::text_from_rgba_image_ref(&image_rgba);
            // cached resized image
            self.set_cache(inner, image_rgba);
            text
        } else {
            let cache = self.cache().lock().unwrap();
            let image = &cache.as_ref().unwrap().image;
            Self::text_from_rgba_image_ref(image)
        };
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

    let color_up = blend_with_background(color_up, position, 0);
    let color_down = blend_with_background(color_down, position, 1);

    let color_up = convert_image_color_to_ratatui_color(color_up);
    let color_down = convert_image_color_to_ratatui_color(color_down);

    let style = Style::default().fg(color_up).bg(color_down);
    Span::styled(String::from(PIXEL), style)
}

fn blend_with_background(
    color: impl Into<Rgba<u8>>,
    position: (usize, usize),
    offset: usize,
) -> Rgba<u8> {
    let color = color.into();
    if color[3] == 255 {
        color
    } else {
        let is_white = (position.0 + position.1 * 2 + offset) % 2 == 0;
        let mut base = if is_white { WHITE } else { GRAY };
        base.blend(&color);
        base
    }
}
fn convert_image_color_to_ratatui_color(color: Rgba<u8>) -> Color {
    Color::Rgb(color[0], color[1], color[2])
}
