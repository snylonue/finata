use crate::{Config, Error, Extract, FinaResult};

pub mod bilibili;
pub mod netease_music;
pub mod pixiv;

pub trait Extractor: Extract + Config {}

impl<T: Extract + Config> Extractor for T {}

pub fn choose_extractor(url: &str) -> FinaResult<Box<dyn Extractor + 'static>> {
    let url = url::Url::parse(url)?;
    // todo: deal with unsupported url properly
    match url.domain() {
        Some("bilibili.com" | "www.bilibili.com") => {
            if url.as_str().contains("av") || url.as_str().contains("bv") {
                Ok(Box::new(bilibili::Video::new(url.as_str())?))
            } else {
                Ok(Box::new(bilibili::Bangumi::new(url.as_str())?))
            }
        }
        Some("music.163.com") => {
            if url.as_str().contains("song") {
                Ok(Box::new(netease_music::Song::new(url.as_str())?))
            } else {
                Ok(Box::new(netease_music::PlayList::new(url.as_str())?))
            }
        }
        Some("pixiv.net" | "www.pixiv.net") => {
            if url.as_str().contains("artworks") {
                Ok(Box::new(pixiv::Pixiv::new(url.as_str())?))
            } else {
                Ok(Box::new(pixiv::Collection::new(url.as_str())?))
            }
        }
        _ => Err(Error::InvalidUrl { url }),
    }
}
