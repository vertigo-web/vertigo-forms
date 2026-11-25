//! A picture made ready to send, in the browser: [`prepare_image`].

use std::rc::Rc;

use vertigo::{
    DropFileItem,
    image::{Resize, Resized, resize},
};

/// The format of a picture's file, told by how the file starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Png,
    Gif,
    Webp,
    /// None of the above. The browser may read it all the same, f. ex. an AVIF or a BMP.
    Other,
}

impl ImageKind {
    pub fn of(data: &[u8]) -> Self {
        match data {
            [0xff, 0xd8, 0xff, ..] => Self::Jpeg,
            [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => Self::Png,
            [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Self::Gif,
            [
                b'R',
                b'I',
                b'F',
                b'F',
                _,
                _,
                _,
                _,
                b'W',
                b'E',
                b'B',
                b'P',
                ..,
            ] => Self::Webp,
            _ => Self::Other,
        }
    }

    /// As people call it: `JPEG`, `PNG`, `GIF`, `WebP`. Nothing for [`ImageKind::Other`].
    pub fn name(self) -> Option<&'static str> {
        match self {
            Self::Jpeg => Some("JPEG"),
            Self::Png => Some("PNG"),
            Self::Gif => Some("GIF"),
            Self::Webp => Some("WebP"),
            Self::Other => None,
        }
    }
}

/// How [`prepare_image`] makes a picture ready.
#[derive(Clone, Debug, PartialEq)]
pub struct PrepareImage {
    /// The longest a side of the picture may be, in pixels. A bigger picture is scaled down,
    /// keeping its proportions.
    pub max_side: u32,
    /// The quality of the JPEG made, from 0, the smallest file, to 1.
    pub quality: f32,
    /// The formats left as they are when the picture fits `max_side`: the file goes on
    /// untouched. A picture of another format becomes a JPEG even when it fits.
    pub keep: Vec<ImageKind>,
}

impl Default for PrepareImage {
    /// At most 1600 pixels a side, a JPEG of quality 0.85, a JPEG that fits left as it is.
    fn default() -> Self {
        Self {
            max_side: 1600,
            quality: 0.85,
            keep: vec![ImageKind::Jpeg],
        }
    }
}

/// A picture [`prepare_image`] made ready, and what became of it, for the app to tell the user
/// in its own words.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedImage {
    /// The file to send: the one given, or a JPEG made of it, named as the one given with
    /// `.jpg` for its extension.
    pub file: DropFileItem,
    /// The format of the file given.
    pub original_kind: ImageKind,
    /// The width and the height of the picture given, turned as its camera wrote into it.
    pub original_size: (u32, u32),
    /// The width and the height of the picture in `file`.
    pub size: (u32, u32),
    /// Whether `file` is a JPEG made here, not the file given.
    pub encoded: bool,
}

impl PreparedImage {
    /// Whether the picture was scaled down.
    pub fn resized(&self) -> bool {
        self.size != self.original_size
    }
}

/// The picture in `file` made ready to send, by the browser: scaled down to `how.max_side` when
/// it is bigger, and a JPEG - unless its format is one of `how.keep` and it fits, when the file
/// is left as it is.
///
/// A phone takes photos of several megabytes, a page shows a fraction of that. This is
/// [`vertigo::image::resize`] with what a form wants around it, and like it needs no decoder
/// in the app's wasm: the `image` crate with JPEG, PNG, WebP and GIF is over 400 KB there.
/// A photo is turned as its camera wrote into it (EXIF), as the JPEG made has no such note.
///
/// Fails with what the browser said when it can't read the picture, and when there is no
/// browser (on the server, in tests).
///
/// ```no_run
/// use vertigo::DropFileItem;
/// use vertigo_forms::{PrepareImage, prepare_image};
///
/// async fn picked(file: DropFileItem) -> Result<DropFileItem, String> {
///     let prepared = prepare_image(file, &PrepareImage::default()).await?;
///     if prepared.resized() {
///         let (width, height) = prepared.size;
///         vertigo::log::info!("made {width}x{height}");
///     }
///     Ok(prepared.file)
/// }
/// ```
pub async fn prepare_image(
    file: DropFileItem,
    how: &PrepareImage,
) -> Result<PreparedImage, String> {
    let kind = ImageKind::of(&file.data);
    let resized = resize(file.data.to_vec(), plan(kind, how)).await?;
    Ok(prepared(file, kind, resized))
}

/// What the browser is asked for a picture of `kind`.
fn plan(kind: ImageKind, how: &PrepareImage) -> Resize {
    Resize {
        quality: how.quality,
        keep_fitting: how.keep.contains(&kind),
        ..Resize::fit(how.max_side, how.max_side)
    }
}

fn prepared(file: DropFileItem, original_kind: ImageKind, resized: Resized) -> PreparedImage {
    let original_size = (resized.original_width, resized.original_height);
    let size = (resized.width, resized.height);

    match resized.data {
        Some(data) => PreparedImage {
            file: DropFileItem {
                name: jpg_name(&file.name),
                data: Rc::new(data),
            },
            original_kind,
            original_size,
            size,
            encoded: true,
        },
        None => PreparedImage {
            file,
            original_kind,
            original_size,
            size,
            encoded: false,
        },
    }
}

/// `name` with `.jpg` for its extension, or after it when it has none. A JPEG's own name stays.
fn jpg_name(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => {
            if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
                return name.to_string();
            }
            stem
        }
        _ => name,
    };
    format!("{stem}.jpg")
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use vertigo::{DropFileItem, image::Resized};

    use super::{ImageKind, PrepareImage, jpg_name, plan, prepared};

    fn file(name: &str, data: &[u8]) -> DropFileItem {
        DropFileItem {
            name: name.to_string(),
            data: Rc::new(data.to_vec()),
        }
    }

    #[test]
    fn the_format_by_how_the_file_starts() {
        assert_eq!(
            ImageKind::of(&[0xff, 0xd8, 0xff, 0xe0, 0, 16]),
            ImageKind::Jpeg
        );
        assert_eq!(
            ImageKind::of(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"),
            ImageKind::Png
        );
        assert_eq!(ImageKind::of(b"GIF89a\x01\0"), ImageKind::Gif);
        assert_eq!(ImageKind::of(b"GIF87a\x01\0"), ImageKind::Gif);
        assert_eq!(ImageKind::of(b"RIFF\x24\0\0\0WEBPVP8 "), ImageKind::Webp);
        assert_eq!(ImageKind::of(b"RIFF\x24\0\0\0WAVEfmt "), ImageKind::Other);
        assert_eq!(ImageKind::of(b"BM"), ImageKind::Other);
        assert_eq!(ImageKind::of(&[]), ImageKind::Other);
        assert_eq!(ImageKind::Webp.name(), Some("WebP"));
        assert_eq!(ImageKind::Other.name(), None);
    }

    #[test]
    fn a_format_to_keep_is_kept_when_it_fits() {
        let how = PrepareImage {
            max_side: 800,
            quality: 0.9,
            keep: vec![ImageKind::Jpeg, ImageKind::Gif],
        };

        let jpeg = plan(ImageKind::Jpeg, &how);
        assert!(jpeg.keep_fitting);
        assert_eq!(
            (jpeg.max_width, jpeg.max_height, jpeg.quality),
            (800, 800, 0.9)
        );
        assert!(plan(ImageKind::Gif, &how).keep_fitting);
        // another one becomes a JPEG whatever its size
        assert!(!plan(ImageKind::Png, &how).keep_fitting);
        assert!(!plan(ImageKind::Other, &how).keep_fitting);

        assert_eq!(PrepareImage::default().keep, vec![ImageKind::Jpeg]);
    }

    #[test]
    fn what_became_of_the_picture() {
        let made = prepared(
            file("Holiday.PNG", b"\x89PNG"),
            ImageKind::Png,
            Resized {
                data: Some(vec![0xff, 0xd8, 0xff]),
                width: 1600,
                height: 1200,
                original_width: 4000,
                original_height: 3000,
            },
        );
        assert_eq!(made.file, file("Holiday.jpg", &[0xff, 0xd8, 0xff]));
        assert_eq!(
            (made.original_size, made.size),
            ((4000, 3000), (1600, 1200))
        );
        assert!(made.encoded && made.resized());

        // a PNG that fits: a JPEG of the same size
        let encoded = prepared(
            file("icon.png", b"\x89PNG"),
            ImageKind::Png,
            Resized {
                data: Some(vec![0xff, 0xd8, 0xff]),
                width: 64,
                height: 64,
                original_width: 64,
                original_height: 64,
            },
        );
        assert!(encoded.encoded && !encoded.resized());

        // kept: the file given, untouched
        let kept = prepared(
            file("photo.jpg", &[0xff, 0xd8, 0xff, 1, 2]),
            ImageKind::Jpeg,
            Resized {
                data: None,
                width: 800,
                height: 600,
                original_width: 800,
                original_height: 600,
            },
        );
        assert_eq!(kept.file, file("photo.jpg", &[0xff, 0xd8, 0xff, 1, 2]));
        assert!(!kept.encoded && !kept.resized());
    }

    #[test]
    fn the_name_of_a_jpeg_made() {
        assert_eq!(jpg_name("photo.png"), "photo.jpg");
        assert_eq!(jpg_name("my.holiday.webp"), "my.holiday.jpg");
        assert_eq!(jpg_name("photo.JPG"), "photo.JPG");
        assert_eq!(jpg_name("photo.jpeg"), "photo.jpeg");
        assert_eq!(jpg_name("photo"), "photo.jpg");
        assert_eq!(jpg_name(".hidden"), ".hidden.jpg");
        assert_eq!(jpg_name(""), ".jpg");
    }
}
