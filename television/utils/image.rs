use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Pixel, Rgba};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Color;
use ratatui::widgets::Widget;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

static PIXEL_STRING: &str = "▀";
const FILTER_TYPE: FilterType = FilterType::Lanczos3;

// use to reduce the size of the image before storing it
const CACHED_WIDTH: u32 = 128;
const CACHED_HEIGHT: u32 = 128;

const GRAY: Rgba<u8> = Rgba([242, 242, 242, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

pub struct ImagePreviewWidget {
    displayed_image: Arc<Mutex<Option<DisplayedImage>>>,
}
impl ImagePreviewWidget {
    pub fn new(
        displayed_image: Arc<Mutex<Option<DisplayedImage>>>,
    ) -> ImagePreviewWidget {
        ImagePreviewWidget { displayed_image }
    }
}
impl Widget for ImagePreviewWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.displayed_image
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .render(area, buf);
    }
}

#[derive(Default)]
pub struct DisplayedImage {
    pub area: Rect,
    cells: Vec<Vec<Cell>>,
}

impl Widget for &DisplayedImage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = self.cells.len();
        if height == 0 {
            return;
        }
        let width = self.cells[0].len();
        // offset of the left top corner where the image is centered
        let total_width = usize::from(area.width) + 2 * usize::from(area.x);
        let x_offset = total_width.saturating_sub(width) / 2;
        let total_height = usize::from(area.height) + 2 * usize::from(area.y);
        let y_offset = total_height.saturating_sub(height) / 2;

        let (area_border_up, area_border_down) =
            (area.y, area.y + area.height);
        let (area_border_left, area_border_right) =
            (area.x, area.x + area.width);
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let pos_x = u16::try_from(x_offset + x).unwrap_or(u16::MAX);
                let pos_y = u16::try_from(y_offset + y).unwrap_or(u16::MAX);
                if (pos_y >= area_border_up && pos_y < area_border_down)
                    && (pos_x >= area_border_left && pos_x < area_border_right)
                {
                    if let Some(buf_cell) =
                        buf.cell_mut(Position::new(pos_x, pos_y))
                    {
                        *buf_cell = cell.clone();
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct CachedImageData {
    image: DynamicImage,
    inner_cache: Arc<Mutex<Option<DisplayedImage>>>,
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

    fn cache(&self) -> &Arc<Mutex<Option<DisplayedImage>>> {
        &self.inner_cache
    }
    pub fn set_cache(&self, area: Rect, cells: Vec<Vec<Cell>>) {
        let mut mutex_cache = self.cache().lock().unwrap();
        if let Some(cache) = mutex_cache.as_mut() {
            cache.area = area;
            cache.cells = cells;
        } else {
            *mutex_cache = Some(DisplayedImage { area, cells });
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
    pub fn image_preview_widget(&self, inner: Rect) -> ImagePreviewWidget {
        // if nothing in the cache of the image, or the area has changed, generate a new image to be displayed and cache it
        if self.cache().lock().unwrap().is_none()
            || self.cache().lock().unwrap().as_ref().unwrap().area != inner
        {
            self.set_cache(inner, self.cells_for_area(inner));
        }
        ImagePreviewWidget::new(self.inner_cache.clone())
    }
    pub fn cells_for_area(&self, inner: Rect) -> Vec<Vec<Cell>> {
        // size of the available area
        let preview_width = u32::from(inner.width);
        let preview_height = u32::from(inner.height) * 2; // *2 because 2 pixels per character
                                                          // resize if it doesn't fit in
        let image_rgba = if self.image.width() > preview_width
            || self.image.height() > preview_height
        {
            self.image
                .resize(preview_width, preview_height, FILTER_TYPE)
                .into_rgba8()
        } else {
            self.image.to_rgba8()
        };
        //creation of the grid of cell
        image_rgba
            // iter over pair of rows
            .rows()
            .step_by(2)
            .zip(image_rgba.rows().skip(1).step_by(2))
            .enumerate()
            .map(|(double_row_y, (row_1, row_2))| {
                // create rows of cells
                row_1
                    .into_iter()
                    .zip(row_2)
                    .enumerate()
                    .map(|(x, (color_up, color_down))| {
                        let position = (x, double_row_y);
                        DoublePixel::new(*color_up, *color_down)
                            .add_grid_background(position)
                            .into_cell()
                    })
                    .collect::<Vec<Cell>>()
            })
            .collect::<Vec<Vec<Cell>>>()
    }
}

// util to convert Rgba into ratatui's Cell
struct DoublePixel {
    color_up: Rgba<u8>,
    color_down: Rgba<u8>,
}
impl DoublePixel {
    pub fn new(color_up: Rgba<u8>, color_down: Rgba<u8>) -> Self {
        Self {
            color_up,
            color_down,
        }
    }

    pub fn add_grid_background(mut self, position: (usize, usize)) -> Self {
        let color_up = self.color_up.0;
        let color_down = self.color_down.0;
        self.color_up = Self::blend_with_background(color_up, position, 0);
        self.color_down = Self::blend_with_background(color_down, position, 1);
        self
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

    pub fn into_cell(self) -> Cell {
        let mut cell = Cell::new(PIXEL_STRING);
        cell.set_bg(Self::convert_image_color_to_ratatui_color(
            self.color_down,
        ))
        .set_fg(Self::convert_image_color_to_ratatui_color(self.color_up));
        cell
    }

    fn convert_image_color_to_ratatui_color(color: Rgba<u8>) -> Color {
        Color::Rgb(color[0], color[1], color[2])
    }
}
