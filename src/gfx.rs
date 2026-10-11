//! Software framebuffer, sprite blits, and the pixel font.

pub struct Image {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

pub struct Assets {
    pub hero: Image,
    pub ninja: Image,
    pub yamo: Image,
    pub tiles: Image,
    pub lantern: Image,
    pub torch: Image,
    pub statue: Image,
    pub gate: Image,
    pub hatch: Image,
    pub bg: [Image; 3],
}

impl Assets {
    pub fn load() -> Self {
        Self {
            hero: load(include_bytes!("../assets/hero.png")),
            ninja: load(include_bytes!("../assets/ninja.png")),
            yamo: load(include_bytes!("../assets/yamo.png")),
            tiles: load(include_bytes!("../assets/tiles.png")),
            lantern: load(include_bytes!("../assets/lantern.png")),
            torch: load(include_bytes!("../assets/torch.png")),
            statue: load(include_bytes!("../assets/statue.png")),
            gate: load(include_bytes!("../assets/gate.png")),
            hatch: load(include_bytes!("../assets/hatch.png")),
            bg: [
                load(include_bytes!("../assets/bg0.png")),
                load(include_bytes!("../assets/bg1.png")),
                load(include_bytes!("../assets/bg2.png")),
            ],
        }
    }
}

fn load(bytes: &[u8]) -> Image {
    let img = image::load_from_memory(bytes).expect("png").to_rgba8();
    let w = img.width() as i32;
    let h = img.height() as i32;
    Image { w, h, px: img.into_raw() }
}

pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u32>,
}

impl Frame {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![0; (w * h) as usize] }
    }

    pub fn clear(&mut self, color: u32) {
        self.px.fill(color);
    }

    pub fn pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = color;
        }
    }
}

pub fn blit(frame: &mut Frame, img: &Image, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32, flip: bool, white: bool) {
    for y in 0..sh {
        for x in 0..sw {
            let src_x = if flip { sx + sw - 1 - x } else { sx + x };
            let src_y = sy + y;
            if src_x < 0 || src_y < 0 || src_x >= img.w || src_y >= img.h {
                continue;
            }
            let i = ((src_y * img.w + src_x) * 4) as usize;
            let a = img.px[i + 3];
            if a < 16 {
                continue;
            }
            let mut r = img.px[i];
            let mut g = img.px[i + 1];
            let mut b = img.px[i + 2];
            if white {
                r = 240;
                g = 240;
                b = 245;
            }
            let color = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            frame.pixel(dx + x, dy + y, color);
        }
    }
}

pub fn blit_frame(frame: &mut Frame, img: &Image, cols: i32, cell_w: i32, cell_h: i32, index: i32, feet_x: i32, feet_y: i32, anchor: (i32, i32), flip: bool, white: bool) {
    let sx = (index % cols) * cell_w;
    let sy = (index / cols) * cell_h;
    blit(frame, img, sx, sy, cell_w, cell_h, feet_x - anchor.0, feet_y - anchor.1, flip, white);
}

pub fn rect(frame: &mut Frame, x: i32, y: i32, w: i32, h: i32, color: u32) {
    for yy in y..y + h {
        for xx in x..x + w {
            frame.pixel(xx, yy, color);
        }
    }
}

pub fn blend_rect(frame: &mut Frame, x: i32, y: i32, w: i32, h: i32, color: u32, alpha: u32) {
    let sr = (color >> 16) & 255;
    let sg = (color >> 8) & 255;
    let sb = color & 255;
    for yy in y.max(0)..(y + h).min(frame.h) {
        for xx in x.max(0)..(x + w).min(frame.w) {
            let p = frame.px[(yy * frame.w + xx) as usize];
            let dr = (p >> 16) & 255;
            let dg = (p >> 8) & 255;
            let db = p & 255;
            let r = (sr * alpha + dr * (255 - alpha)) / 255;
            let g = (sg * alpha + dg * (255 - alpha)) / 255;
            let b = (sb * alpha + db * (255 - alpha)) / 255;
            frame.px[(yy * frame.w + xx) as usize] = (r << 16) | (g << 8) | b;
        }
    }
}

/// Bayer-dithered lamp glow. Stays in the pixel-art grid.
pub fn glow(frame: &mut Frame, cx: i32, cy: i32, rad: i32, color: u32) {
    let bayer = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let rad2 = rad * rad;
    for y in cy - rad..cy + rad {
        for x in cx - rad..cx + rad {
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            if d2 > rad2 {
                continue;
            }
            let thresh = d2 * 16 / rad2.max(1);
            if bayer[(y & 3) as usize][(x & 3) as usize] < 16 - thresh {
                let a = 90 - thresh as u32 * 3;
                if x >= 0 && y >= 0 && x < frame.w && y < frame.h {
                    let i = (y * frame.w + x) as usize;
                    let p = frame.px[i];
                    let sr = (color >> 16) & 255;
                    let sg = (color >> 8) & 255;
                    let sb = color & 255;
                    let dr = (p >> 16) & 255;
                    let dg = (p >> 8) & 255;
                    let db = p & 255;
                    let r = (sr * a + dr * (255 - a)) / 255;
                    let g = (sg * a + dg * (255 - a)) / 255;
                    let b = (sb * a + db * (255 - a)) / 255;
                    frame.px[i] = (r << 16) | (g << 8) | b;
                }
            }
        }
    }
}

pub fn text_width(s: &str, scale: i32) -> i32 {
    s.chars().count() as i32 * (5 * scale + scale)
}

pub fn draw_text(frame: &mut Frame, s: &str, mut x: i32, y: i32, color: u32, scale: i32) {
    for ch in s.chars() {
        let g = glyph(ch);
        for row in 0..7 {
            for col in 0..5 {
                if (g[row] >> (4 - col)) & 1 == 1 {
                    let col = col as i32;
                    let row = row as i32;
                    rect(frame, x + col * scale, y + row * scale, scale, scale, color);
                }
            }
        }
        x += 5 * scale + scale;
    }
}

pub fn draw_text_shadow(frame: &mut Frame, s: &str, x: i32, y: i32, color: u32, shadow: u32, scale: i32) {
    draw_text(frame, s, x + scale, y + scale, shadow, scale);
    draw_text(frame, s, x, y, color, scale);
}

fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110],
        '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100],
        '\'' => [0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}
