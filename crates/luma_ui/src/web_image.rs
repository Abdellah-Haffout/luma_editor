use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Central resource managing background downloads and caching of web images.
#[derive(Resource)]
pub struct WebImageLoader {
    cache: HashMap<String, Handle<Image>>,
    in_flight: HashSet<String>,
    failed: HashSet<String>,
    sender: Sender<(String, Result<Image, String>)>,
    receiver: Arc<Mutex<Receiver<(String, Result<Image, String>)>>>,
    pub loading_texture: Handle<Image>,
    pub error_texture: Handle<Image>,
}

impl WebImageLoader {
    pub fn new(images: &mut Assets<Image>) -> Self {
        let (sender, receiver) = channel();
        let loading_texture = create_procedural_placeholder(images, [30, 41, 59, 255], [56, 189, 248, 255]); // Slate & Sky Blue
        let error_texture = create_procedural_placeholder(images, [69, 26, 26, 255], [239, 68, 68, 255]);   // Dark Red & Coral

        Self {
            cache: HashMap::new(),
            in_flight: HashSet::new(),
            failed: HashSet::new(),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            loading_texture,
            error_texture,
        }
    }

    /// Returns the cached texture handle if the URL has already been downloaded.
    pub fn get(&self, url: &str) -> Option<Handle<Image>> {
        self.cache.get(url).cloned()
    }

    /// Dispatches a background thread to fetch the image if not already cached, in-flight, or failed.
    /// Returns `true` if a new download was started.
    pub fn request(&mut self, url: &str) -> bool {
        let trimmed = url.trim();
        if trimmed.is_empty()
            || self.cache.contains_key(trimmed)
            || self.in_flight.contains(trimmed)
            || self.failed.contains(trimmed)
        {
            return false;
        }

        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return false;
        }

        self.in_flight.insert(trimmed.to_string());
        let sender = self.sender.clone();
        let target_url = trimmed.to_string();

        std::thread::Builder::new()
            .name(format!("web-img-{}", trimmed))
            .spawn(move || {
                let result = fetch_and_decode_image(&target_url);
                let _ = sender.send((target_url, result));
            })
            .ok();

        true
    }

    /// Gets the cached texture handle, or starts downloading it in the background
    /// and returns `None` (or fallback) while in flight.
    pub fn get_or_request(&mut self, url: &str) -> Option<Handle<Image>> {
        let trimmed = url.trim();
        if let Some(handle) = self.cache.get(trimmed) {
            return Some(handle.clone());
        }

        self.request(trimmed);

        if self.failed.contains(trimmed) {
            Some(self.error_texture.clone())
        } else if self.in_flight.contains(trimmed) {
            Some(self.loading_texture.clone())
        } else {
            None
        }
    }

    pub fn is_in_flight(&self, url: &str) -> bool {
        self.in_flight.contains(url.trim())
    }

    pub fn has_failed(&self, url: &str) -> bool {
        self.failed.contains(url.trim())
    }

    pub fn retry(&mut self, url: &str) {
        let trimmed = url.trim();
        self.failed.remove(trimmed);
        self.request(trimmed);
    }
}

/// Internal worker to download image bytes over HTTP/HTTPS and decode into a Bevy `Image`.
fn fetch_and_decode_image(url: &str) -> Result<Image, String> {
    let response = ureq::get(url)
        .set("User-Agent", "LumaEditor/0.1.0 (Bevy Engine)")
        .set("Accept", "image/png,image/jpeg,image/webp,image/*,*/*;q=0.8")
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(20 * 1024 * 1024) // 20 MB max limit
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let dynamic_img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image data: {e}"))?;

    let rgba = dynamic_img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw_bytes = rgba.into_raw();

    Ok(Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        raw_bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

/// Generates a simple 64x64 checkerboard placeholder texture.
fn create_procedural_placeholder(
    images: &mut Assets<Image>,
    color_a: [u8; 4],
    color_b: [u8; 4],
) -> Handle<Image> {
    let size = 64u32;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let check = ((x / 8) + (y / 8)) % 2 == 0;
            if check {
                data.extend_from_slice(&color_a);
            } else {
                data.extend_from_slice(&color_b);
            }
        }
    }
    images.add(Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

fn setup_web_image_loader(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(WebImageLoader::new(&mut images));
}

/// Polls background download results and uploads completed images to Bevy's asset storage.
pub fn update_web_image_loader(
    loader: Option<ResMut<WebImageLoader>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut loader) = loader else { return };
    let items: Vec<(String, Result<Image, String>)> = {
        let Ok(receiver) = loader.receiver.lock() else { return };
        let mut results = Vec::new();
        while let Ok(item) = receiver.try_recv() {
            results.push(item);
        }
        results
    };

    for (url, result) in items {
        loader.in_flight.remove(&url);
        match result {
            Ok(img) => {
                let handle = images.add(img);
                loader.cache.insert(url, handle);
            }
            Err(err) => {
                warn!("[WebImageLoader] Error loading '{}': {}", url, err);
                loader.failed.insert(url);
            }
        }
    }
}

pub struct WebImagePlugin;

impl Plugin for WebImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_web_image_loader)
            .add_systems(Update, update_web_image_loader);
    }
}
