use image::{ImageResult, Rgba, Rgba32FImage, RgbaImage};

pub(crate) struct Image {
    data: Rgba32FImage,
    depth_buf: Vec<Option<f64>>,
    frame_buf: Vec<Rgba<f32>>,
    width: u32,
    height: u32,
}

impl Image {
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
        }
    }

    pub(crate) fn from_pixel(width: u32, height: u32, pixel: Rgba<f32>) -> Self {
        let data = Rgba32FImage::from_pixel(width, height, pixel);
        let depth_buf = vec![None; (width * height) as usize];
        let frame_buf = vec![Rgba([0f32, 0f32, 0f32, 0f32]); (width * height) as usize];

        Self {
            data,
            depth_buf,
            frame_buf,
            width,
            height,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.data.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.data.height()
    }

    pub(crate) fn depth(&self, x: u32, y: u32) -> f64 {
        let coord = (y * self.width + x) as usize;
        self.depth_buf[coord].unwrap_or(f64::MAX)
    }

    pub(crate) fn put_pixel(&mut self, x: u32, y: u32, pixel: Rgba<f32>, depth: Option<f64>) {
        let coord = (y * self.width + x) as usize;
        self.frame_buf[coord] = pixel;
        if depth.is_some() {
            self.depth_buf[coord] = depth;
        }
    }

    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> Rgba<f32> {
        let coord = (y * self.width + x) as usize;
        return self.frame_buf[coord];
    }

    fn save_data(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let coord = (y * self.width + x) as usize;
                self.data.put_pixel(x, y, self.frame_buf[coord]);
            }
        }
    }

    pub(crate) fn save(&mut self, path: String) -> ImageResult<()> {
        self.save_data();
        let temp: RgbaImage = RgbaImage::from_vec(
            self.width(),
            self.height(),
            self.data.clone().into_vec().iter().map(|&a| a as u8).collect(),
        )
        .unwrap();
        temp.save(path)
    }
}
