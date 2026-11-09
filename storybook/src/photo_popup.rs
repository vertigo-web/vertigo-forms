use vertigo::{DomNode, css, dom};
use vertigo_forms::{Photo, PhotoGallery, PhotoGalleryParams, PhotoPopupParams};

/// A picture of `width`×`height` with `label` in the middle, so the story needs no network.
fn sample(label: &str, color: &str, width: u32, height: u32) -> Photo {
    Photo {
        url: format!(
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'%3E\
             %3Crect width='100%25' height='100%25' fill='{color}'/%3E\
             %3Ctext x='50%25' y='50%25' font-size='{size}' fill='white' text-anchor='middle' dominant-baseline='middle'%3E{label}%3C/text%3E\
             %3C/svg%3E",
            size = height / 4,
        ),
        alt: format!("Photo {label}"),
    }
}

pub fn photo_popup() -> DomNode {
    let photos = vec![
        sample("1", "steelblue", 1200, 800),
        sample("2", "indianred", 600, 900),
        sample("3", "seagreen", 800, 800),
    ];

    let single = vec![sample("4", "darkorange", 1600, 600)];
    let custom = PhotoGalleryParams {
        thumbnail_css: css! {"
            height: 100px;
            width: auto;
            border: 2px solid gray;
        "},
        popup: PhotoPopupParams {
            close_label: "Close photo".to_string(),
            previous_label: "Previous photo".to_string(),
            next_label: "Next photo".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    dom! {
        <div>
            <p>"Gallery:"</p>
            <PhotoGallery {photos} params={} t:loading="lazy" />
            <p>"One photo, custom labels:"</p>
            <PhotoGallery photos={single} params={custom} />
        </div>
    }
}
