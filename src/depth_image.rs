use image::{ImageResult, Rgba, Rgba32FImage, RgbaImage};
use palette::rgb::Rgb;
use palette::Srgb;

pub(crate) struct DepthImage {
    data: Rgba32FImage,
    depth_buf: Vec<Option<f64>>,
    frame_buf: Vec<Rgba<f32>>,
    width: u32,
    height: u32,
    fsaa: u32,
}

impl DepthImage {
    pub(crate) fn default() -> Self {
        let data = Rgba32FImage::default();
        let depth_buf = Vec::<Option<f64>>::default();
        let frame_buf = Vec::<Rgba<f32>>::default();

        Self {
            data,
            depth_buf,
            frame_buf,
            width: 0,
            height: 0,
            fsaa: 1,
        }
    }

    pub(crate) fn from_pixel(width: u32, height: u32, pixel: Rgba<f32>, fsaa: u32) -> Self {
        let data = Rgba32FImage::from_pixel(width, height, pixel);
        let depth_buf = vec![None; (width * height * fsaa.pow(2)) as usize];
        let frame_buf = vec![Rgba([0f32; 4]); (width * height * fsaa.pow(2)) as usize];

        Self {
            data,
            depth_buf,
            frame_buf,
            width,
            height,
            fsaa,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.data.width() * self.fsaa
    }

    pub(crate) fn height(&self) -> u32 {
        self.data.height() * self.fsaa
    }

    pub(crate) fn depth(&self, x: u32, y: u32) -> f64 {
        let coord = (y * self.width() + x) as usize;
        self.depth_buf[coord].unwrap_or(f64::MAX)
    }

    pub(crate) fn put_pixel(&mut self, x: u32, y: u32, pixel: Rgba<f32>, depth: Option<f64>) {
        let coord = (y * self.width() + x) as usize;
        self.frame_buf[coord] = pixel;
        if depth.is_some() {
            self.depth_buf[coord] = depth;
        }
    }

    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> Rgba<f32> {
        let coord = (y * self.width() + x) as usize;
        return self.frame_buf[coord];
    }

    fn save_data(&mut self, s_rgb: bool) {
        for x in 0..self.width {
            for y in 0..self.height {
                let mut avg_r = 0f32;
                let mut avg_g = 0f32;
                let mut avg_b = 0f32;
                let mut divisor = 0f32;
                for i in 0..self.fsaa {
                    for j in 0..self.fsaa {
                        let coord = ((y * self.fsaa + j) * self.width() + x * self.fsaa + i) as usize;
                        let pixel = self.frame_buf[coord];

                        avg_r += pixel[0] * pixel[3];
                        avg_g += pixel[1] * pixel[3];
                        avg_b += pixel[2] * pixel[3];
                        divisor += pixel[3];
                    }
                }

                if divisor != 0f32 {
                    avg_r /= divisor;
                    avg_g /= divisor;
                    avg_b /= divisor;
                }
                let avg_a = divisor / self.fsaa.pow(2) as f32;

                if s_rgb {
                    let s_rgb_pixel = Srgb::<f32>::from_linear(Rgb::from_components((avg_r, avg_g, avg_b)));
                    avg_r = s_rgb_pixel.red;
                    avg_g = s_rgb_pixel.green;
                    avg_b = s_rgb_pixel.blue;
                }

                self.data.put_pixel(x, y, Rgba([avg_r, avg_g, avg_b, avg_a]));
            }
        }
    }

    pub(crate) fn save(&mut self, path: String, s_rgb: bool) -> ImageResult<()> {
        self.save_data(s_rgb);
        let temp: RgbaImage = RgbaImage::from_vec(
            self.width,
            self.height,
            self.data
                .clone()
                .into_vec()
                .iter()
                .map(|&a| (a * 255f32) as u8)
                .collect(),
        )
        .unwrap();
        temp.save(path)
    }
}
