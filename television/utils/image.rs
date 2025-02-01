use image::DynamicImage;
use image::imageops::FilterType;

pub const PIXEL: char = '▀' ;
const FILTER: FilterType = FilterType::Triangle;
#[derive(Clone, Debug)]
pub struct Image {
    pub pixel_grid: Vec<Vec<(ImageColor, ImageColor)>>
}
impl Image {
    pub fn new(pixel_grid: Vec<Vec<(ImageColor, ImageColor)>>) -> Self {
        Image { pixel_grid}
    }
    pub fn from_dynamic_image(dynamic_image: DynamicImage, height: u32, width: u32) -> Self {

        let image = if dynamic_image.height() > height || dynamic_image.width() > width {
            println!("{}", dynamic_image.height());
            dynamic_image.resize(width, height, FILTER)
        }else{
            dynamic_image
        };

        let image = image.into_rgba8();
        let pixel_grid = image.rows()
            .step_by(2)
            .zip(image.rows().skip(1).step_by(2))
            .map(|(row_1, row_2)| {
                row_1.zip(row_2)
                    .map(|(pixel_1, pixel_2)|
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
                            }))
                    .collect::<Vec<(ImageColor,ImageColor)>>()
            })
            .collect::<Vec<Vec<(ImageColor,ImageColor)>>>();
        Image::new(pixel_grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ImageColor{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}