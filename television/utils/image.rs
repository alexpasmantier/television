use image::imageops::FilterType;
use image::{DynamicImage, Pixel, Rgba};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Color;
use ratatui::widgets::Widget;
use std::fmt::Debug;
use std::hash::Hash;

static PIXEL_STRING: &str = "▀";
const FILTER_TYPE: FilterType = FilterType::Triangle;

// use to reduce the size of the image before storing it
const DEFAULT_CACHED_WIDTH: u32 = 50;
const DEFAULT_CACHED_HEIGHT: u32 = 100;

const GRAY: Rgba<u8> = Rgba([242, 242, 242, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ImagePreviewWidget {
    cells: Vec<Vec<Cell>>,
}

impl Widget for &ImagePreviewWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = self.height();
        let width = self.width();
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
            let pos_y = u16::try_from(y_offset + y).unwrap_or(u16::MAX);
            if pos_y >= area_border_up && pos_y < area_border_down {
                for (x, cell) in row.iter().enumerate() {
                    let pos_x =
                        u16::try_from(x_offset + x).unwrap_or(u16::MAX);
                    if pos_x >= area_border_left && pos_x < area_border_right {
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
}
impl ImagePreviewWidget {
    pub fn new(cells: Vec<Vec<Cell>>) -> ImagePreviewWidget {
        ImagePreviewWidget { cells }
    }

    pub fn height(&self) -> usize {
        self.cells.len()
    }
    pub fn width(&self) -> usize {
        if self.height() > 0 {
            self.cells[0].len()
        } else {
            0
        }
    }

    pub fn from_dynamic_image(
        dynamic_image: DynamicImage,
        dimension: Option<(u32, u32)>,
    ) -> Self {
        // first quick resize
        let (window_width, window_height) =
            dimension.unwrap_or((DEFAULT_CACHED_WIDTH, DEFAULT_CACHED_HEIGHT));
        let big_resized_image = if dynamic_image.width() > window_width * 4
            || dynamic_image.height() > window_height * 4
        {
            dynamic_image.resize(
                window_width * 4,
                window_height * 4,
                FilterType::Nearest,
            )
        } else {
            dynamic_image
        };
        // this time resize with the filter
        let resized_image = if big_resized_image.width() > window_width
            || big_resized_image.height() > window_height
        {
            big_resized_image.resize(
                window_width,
                DEFAULT_CACHED_HEIGHT,
                FILTER_TYPE,
            )
        } else {
            big_resized_image
        };

        let cells = Self::cells_from_dynamic_image(resized_image);
        ImagePreviewWidget::new(cells)
    }

    fn cells_from_dynamic_image(image: DynamicImage) -> Vec<Vec<Cell>> {
        let image_rgba = image.into_rgba8();

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
