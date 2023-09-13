use image::{ImageResult, Rgba, RgbaImage};

pub(crate) struct Image {
    data: RgbaImage,
    depth_buf: Vec<Option<f64>>,
    width: u32,
}

impl Image {
    pub(crate) fn default() -> Self {
        let data = RgbaImage::default();
        let depth_buf = Vec::<Option<f64>>::default();

        Self { data, depth_buf, width: 0 }
    }

    pub(crate) fn from_pixel(width: u32, height: u32, pixel: Rgba<u8>) -> Self {
        let data = RgbaImage::from_pixel(width, height, pixel);
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

    pub(crate) fn put_pixel(&mut self, x: u32, y: u32, pixel: Rgba<u8>, depth: Option<f64>) {
        self.data.put_pixel(x, y, pixel);
        if depth.is_some() {
            self.depth_buf[(y * self.width + x) as usize] = depth;
        }
    }

    pub(crate) fn save(&self, path: String) -> ImageResult<()> {
        self.data.save(path)
    }
}