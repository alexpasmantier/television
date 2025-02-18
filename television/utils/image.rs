use std::cmp;
use std::cmp::max;
use std::hash::{Hash, Hasher};


use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Luma, Rgb, RgbImage, Rgba, RgbaImage};

use fast_image_resize::{ ImageBufferError, ImageView, IntoImageView, PixelType, ResizeAlg, ResizeOptions, Resizer};
use fast_image_resize::images::Image;
use fast_image_resize::pixels::{Pixel, U8x3};
use image::buffer::ConvertBuffer;
use image::imageops;
use image::imageops::FilterType;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Span, Style, Text};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use tracing::{debug, trace, warn};
pub const PIXEL: char = '▀';
pub const PIXEL_TYPE: PixelType = PixelType::U8x4;
const RESIZE_ALGORITHM: ResizeAlg = ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3);

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
        /*
        let (new_width, new_height) = calculate_fit_dimensions(dynamic_image.width(), dynamic_image.height(), CACHED_WIDTH,CACHED_HEIGHT);

        // fixme resize even if useless and should just change the type
        let mut dst_image = Image::new(
                new_width,
                new_height,
                dynamic_image.pixel_type()?,
        );

        let resize_option = ResizeOptions::new().resize_alg(ResizeAlg::Nearest);
        let mut resizer = Resizer::new();
        resizer.resize(&dynamic_image, &mut dst_image, Some(&resize_option)).unwrap();


        // convert the resized image into rgba
        let rgba_image: RgbaImage = match dst_image.pixel_type() {
            PixelType::U8x3 => {
                let rgb_image: RgbImage = RgbImage::from_raw(new_width, new_height, dst_image.into_vec()).unwrap();
                rgb_image.convert()
            }
            PixelType::U16 => {
                // Convert `Luma<u16>` to `Rgba<u8>` (downscaling 16-bit to 8-bit)
                let rgba_pixels: Vec<u8> =  dst_image.into_vec().chunks_exact(2).flat_map(|b| {
                    let luma16 = u16::from_le_bytes([b[0], b[1]]);
                    let luma8 = (luma16 >> 8) as u8; // Downscale 16-bit to 8-bit
                    vec![luma8, luma8, luma8, 255]
                }).collect();
                ImageBuffer::from_raw(new_width, new_height, rgba_pixels)
                    .expect("Failed to create Rgba8 ImageBuffer from Luma16")
            }
            PixelType::U8x4 => {
                // Directly use the buffer since it's already RGBA8
                ImageBuffer::from_raw(new_width, new_height, dst_image.into_vec())
                    .expect("Failed to create Rgba8 ImageBuffer from U8x4")
            }
            _ => panic!("Unsupported pixel type"),
        };
         */
        let resized_image = if dynamic_image.width() > CACHED_WIDTH || dynamic_image.height() > CACHED_HEIGHT {
            dynamic_image.resize(CACHED_WIDTH, CACHED_HEIGHT , FilterType::Nearest)
        } else {
            dynamic_image
        };
        let rgba_image = resized_image.into_rgba8();
        CachedImageData::new(DynamicImage::from(rgba_image))
    }
    pub fn paragraph<'a>(&self, inner: Rect, preview_block: Block<'a>) -> Paragraph<'a> {
        // resize it for the preview window
        //let mut data = self.image.clone().into_raw();
        //let src_image = DynamicImage::from(self.image.clone());
        /*
        let src_image = if let Some(src_image) = Image::from_slice_u8(self.image.width(), self.image.height(), &mut data, PixelType::U8x4)
            .map_err(|error| warn!("Failed to resize cached image: {error}"))
            .ok(){
            src_image
        } else {
            return Paragraph::new(Text::raw("Failed to resize cached image"));
        };

        let (new_width, new_height) = calculate_fit_dimensions(self.image.width(), self.image.height(), u32::from(inner.width), u32::from(inner.height*2));
        let mut dst_image = Image::new(
            new_width,
            new_height,
            PixelType::U8x4,
        );
        let resize_option = ResizeOptions::new().resize_alg(RESIZE_ALGORITHM);
        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dst_image, &Some(resize_option)).unwrap();

        let image_rgba: RgbaImage = ImageBuffer::from_raw( dst_image.width(), dst_image.height(), dst_image.into_vec()).unwrap();
        */
        let preview_width = u32::from(inner.width);
        let preview_height = u32::from(inner.height) * 2;
        let image_rgba = if self.image.width() > preview_width || self.image.height() > preview_height {
            self.image.resize(preview_width, preview_height, FilterType::Triangle).to_rgba8()
        } else{
            self.image.to_rgba8()
        };
        // transform it into text
        let lines = image_rgba
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

/*
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ImageDoublePixel {
    pub pixel_grid: Vec<Vec<(ImageColor, ImageColor)>>,
}
impl ImageDoublePixel {
    pub fn new(pixel_grid: Vec<Vec<(ImageColor, ImageColor)>>) -> Self {
        ImageDoublePixel { pixel_grid }
    }
    pub fn  from_cached_image(cached_image_data: &CachedImageData,
                               new_width: u32,
                               new_height: u32) -> Option<Self>{
        let mut binding = cached_image_data.data.clone();
        let src_image = Image::from_slice_u8(cached_image_data.width, cached_image_data.height, &mut binding, cached_image_data.pixel_type)
            .map_err(|error| warn!("Failed to resize cached image: {error}"))
            .ok()?;
        let (new_width, new_height) = calculate_fit_dimensions(cached_image_data.width, cached_image_data.height, new_width, new_height);
        let mut dst_image = Image::new(
            new_width,
            new_height,
            cached_image_data.pixel_type,
        );
        let resize_option = ResizeOptions::new().resize_alg(RESIZE_ALGORITHM);
        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dst_image, &Some(resize_option)).unwrap();


        match cached_image_data.pixel_type {
            PixelType::U8x4  => {
                let rgba_image = ImageBuffer::from_raw(new_width, new_height, dst_image.into_vec())?;
                Some(Self::from_rgba_image(rgba_image))
            },
            _ => {
                warn!("Unsupported pixel type: {:?}", cached_image_data.pixel_type);
                println!("Unsupported pixel type: {:?}", cached_image_data.pixel_type);
                None
            }
        }

    }

    pub fn from_rgba_image(
        image: RgbaImage
    ) -> Self {
        let pixel_grid = image
            .rows()
            .step_by(2)
            .zip(image.rows().skip(1).step_by(2))
            .map(|(row_1, row_2)| {
                row_1
                    .zip(row_2)
                    .map(|(pixel_1, pixel_2)| {
                        (
                            ImageColor {
                                r: pixel_1.0[0],
                                g: pixel_1.0[1],
                                b: pixel_1.0[2],
                                a: pixel_1.0[3],
                            },
                            ImageColor {
                                r: pixel_2.0[0],
                                g: pixel_2.0[1],
                                b: pixel_2.0[2],
                                a: pixel_1.0[3],
                            },
                        )
                    })
                    .collect::<Vec<(ImageColor, ImageColor)>>()
            })
            .collect::<Vec<Vec<(ImageColor, ImageColor)>>>();
        ImageDoublePixel::new(pixel_grid)
    }


}
 */
/*
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct ImageColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl ImageColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        ImageColor { r, g, b, a }
    }
    pub const BLACK: ImageColor = ImageColor {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: ImageColor = ImageColor {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const GRAY: ImageColor = ImageColor {
        r: 242,
        g: 242,
        b: 242,
        a: 255,
    };
}
*/


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

    let alpha_threshold = 30;
    let color_up = if color_up[3] <= alpha_threshold {
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

    let color_up =     convert_image_color_to_ratatui_color(color_up);
    let color_down = convert_image_color_to_ratatui_color(color_down);


    let style = Style::default().fg(color_up).bg(color_down);
    Span::styled(String::from(PIXEL), style)
}

fn convert_image_color_to_ratatui_color(color: Rgba<u8>) -> Color {
    Color::Rgb(color[0], color[1], color[2])
}


