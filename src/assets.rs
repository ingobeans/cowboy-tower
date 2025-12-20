use std::collections::HashMap;

use asefile::AsepriteFile;
use image::EncodableLayout;
use include_dir::{Dir, include_dir};
use macroquad::prelude::*;

use crate::utils::create_camera;

pub struct Assets {
    pub cowboy: AnimationsGroup,
    pub levels: Vec<Level>,
    pub tileset: Spritesheet,
}
impl Assets {
    pub fn load() -> Self {
        let tileset = Spritesheet::new(
            load_ase_texture(include_bytes!("../assets/tileset.ase"), None),
            8.0,
        );

        let mut levels = Vec::new();
        static LEVELS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/levels");
        for file in LEVELS_DIR.files() {
            let level = Level::load(file.contents_utf8().unwrap(), &tileset);
            levels.push(level);
        }
        Self {
            levels,
            cowboy: AnimationsGroup::from_file(include_bytes!("../assets/cowboy.ase")),
            tileset,
        }
    }
}

pub struct Level {
    pub width: usize,
    pub data: Vec<[u8; 3]>,
    pub camera: Camera2D,
}
impl Level {
    pub fn get_tile(&self, x: usize, y: usize) -> [u8; 3] {
        self.data[x + y * self.width]
    }
    pub fn load(data: &str, tileset: &Spritesheet) -> Self {
        let mut layers = data.split("<layer");
        layers.next();
        let first = layers.next().unwrap();
        let first_chunks = get_all_chunks(first);
        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;
        for (x, y) in first_chunks.keys() {
            if *x < min_x {
                min_x = *x;
            } else if *x > max_x {
                max_x = *x;
            }
            if *y < min_y {
                min_y = *y;
            } else if *y > max_y {
                max_y = *y;
            }
        }
        let width = max_x - min_x + 16;
        let height = max_y - min_y + 16;
        dbg!(width, height);
        let mut data = vec![[0, 0, 0]; (width * height) as usize];

        for (index, chunks) in [first_chunks]
            .into_iter()
            .chain(layers.map(|f| get_all_chunks(f)))
            .enumerate()
        {
            for ((cx, cy), chunk) in chunks.iter() {
                for (i, tile) in chunk.tiles.iter().enumerate() {
                    let x = (i % 16) + (*cx - min_x) as usize;
                    let y = (i / 16) + (*cy - min_y) as usize;
                    data[x + y * width as usize][index] = *tile;
                }
            }
        }
        let mut camera = create_camera((width * 8) as f32, (height * 8) as f32);
        camera.target = vec2((width * 8) as f32 / 2.0, (height * 8) as f32 / 2.0);
        set_camera(&camera);
        for (i, tile) in data.iter().enumerate() {
            let x = i % width as usize;
            let y = i / width as usize;
            for t in tile {
                if *t == 0 {
                    continue;
                }
                let t = *t - 1;
                tileset.draw_tile(
                    (x * 8) as f32,
                    (y * 8) as f32,
                    (t % 32) as f32,
                    (t / 32) as f32,
                    None,
                );
            }
        }
        set_default_camera();
        Self {
            width: width as usize,
            camera,
            data,
        }
    }
}
#[derive(Clone)]
pub struct Chunk {
    pub x: i16,
    pub y: i16,
    pub tiles: Vec<u8>,
}
impl Chunk {
    pub fn tile_at(&self, x: usize, y: usize) -> Option<u8> {
        if x > 16 || y > 16 {
            return None;
        }
        self.tiles.get(x + y * 16).cloned()
    }
}

fn get_all_chunks(xml: &str) -> HashMap<(i16, i16), Chunk> {
    let mut chunks = HashMap::new();
    let mut xml = xml.to_string();
    while let Some((current, remains)) = xml.split_once("</chunk>") {
        let new = parse_chunk(current);
        chunks.insert((new.x, new.y), new);
        xml = remains.to_string();
    }

    chunks
}

fn get_layer<'a>(xml: &'a str, layer: &str) -> &'a str {
    let split = format!(" name=\"{layer}");
    xml.split_once(&split)
        .unwrap()
        .1
        .split_once(">")
        .unwrap()
        .1
        .split_once("</layer>")
        .unwrap()
        .0
}

fn parse_chunk(xml: &str) -> Chunk {
    let (tag, data) = xml
        .split_once("<chunk ")
        .unwrap()
        .1
        .split_once(">")
        .unwrap();

    let x = tag
        .split_once("x=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();
    let y = tag
        .split_once("y=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();

    let mut split = data.split(',');

    let mut chunk = vec![0; 16 * 16];
    for item in &mut chunk {
        let a = split.next().unwrap().trim();
        *item = a.parse().unwrap()
    }
    Chunk { x, y, tiles: chunk }
}

pub struct Animation {
    pub frames: Vec<(Texture2D, u32)>,
    pub total_length: u32,
}
impl Animation {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        let mut total_length = 0;
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            total_length += duration;
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        Self {
            frames,
            total_length,
        }
    }
    pub fn get_at_time(&self, mut time: u32) -> &Texture2D {
        time %= self.total_length;
        for (texture, length) in self.frames.iter() {
            if time >= *length {
                time -= length;
            } else {
                return texture;
            }
        }
        panic!()
    }
}

pub struct AnimationsGroup {
    #[expect(dead_code)]
    pub file: AsepriteFile,
    pub animations: Vec<Animation>,
    pub tag_names: HashMap<String, usize>,
}
impl AnimationsGroup {
    #[expect(dead_code)]
    pub fn get_by_name(&self, name: &str) -> &Animation {
        &self.animations[*self.tag_names.get(name).unwrap()]
    }
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        let mut tag_frames = Vec::new();
        let mut offset = 0;

        let mut tag_names = HashMap::new();

        for i in 0..ase.num_tags() {
            let tag = ase.get_tag(i).unwrap();
            tag_names.insert(tag.name().to_string(), i as usize);
            let (start, end) = (tag.from_frame() as usize, tag.to_frame() as usize);
            let mut total_length = 0;
            let included_frames: Vec<(Texture2D, u32)> = frames
                .extract_if((start - offset)..(end - offset + 1), |_| true)
                .collect();
            for f in included_frames.iter() {
                total_length += f.1;
            }
            offset += end.abs_diff(start) + 1;
            tag_frames.push(Animation {
                frames: included_frames,
                total_length,
            });
        }
        Self {
            file: ase,
            animations: tag_frames,
            tag_names,
        }
    }
}
fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}

pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
        }
    }
    #[expect(dead_code)]
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}
