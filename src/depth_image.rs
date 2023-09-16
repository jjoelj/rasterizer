use image::buffer::ConvertBuffer;
use image::{ImageResult, Rgba, Rgba32FImage, RgbaImage};

pub(crate) struct Image {
    data: Rgba32FImage,
    depth_buf: Vec<Option<f64>>,
    width: u32,
}

impl Image {
    pub(crate) fn default() -> Self {
        let data = Rgba32FImage::default();
        let depth_buf = Vec::<Option<f64>>::default();

        Self {
            data,
            depth_buf,
            width: 0,
        }
    }

    pub(crate) fn from_pixel(width: u32, height: u32, pixel: Rgba<f32>) -> Self {
        let data = Rgba32FImage::from_pixel(width, height, pixel);
        let depth_buf = vec![None; (width * height) as usize];

        Self { data, depth_buf, width }
    }

    pub(crate) fn width(&self) -> u32 {
        self.data.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.data.height()
    }

    pub(crate) fn depth(&self, x: u32, y: u32) -> f64 {
        self.depth_buf[(y * self.width + x) as usize].unwrap_or(f64::MAX)
    }

    pub(crate) fn put_pixel(&mut self, x: u32, y: u32, pixel: Rgba<f32>, depth: Option<f64>) {
        self.data.put_pixel(x, y, pixel);
        if depth.is_some() {
            self.depth_buf[(y * self.width + x) as usize] = depth;
        }
    }

    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> &Rgba<f32> {
        self.data.get_pixel(x, y)
    }

    pub(crate) fn save(&self, path: String) -> ImageResult<()> {
        let temp: RgbaImage = RgbaImage::from_vec(
            self.width(),
            self.height(),
            self.data.clone().into_vec().iter().map(|&a| a as u8).collect(),
        )
        .unwrap();
        temp.save(path)
    }
}
