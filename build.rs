use std::{fs::File, path::Path};

use ico::IconDirEntry;
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree, TreeParsing},
};
#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    let svg = std::fs::read_to_string("logo.svg").unwrap();

    let tree = Tree::from_str(&svg, &Options::default()).unwrap();

    let mut pixmap = Pixmap::new(16, 16).unwrap();

    resvg::render(
        &tree,
        resvg::FitTo::Size(16, 16),
        Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    std::fs::write(
        Path::new(std::env::var_os("OUT_DIR").as_ref().unwrap()).join("logo.bitmap"),
        pixmap.data(),
    )
    .unwrap();
    pixmap
        .save_png(Path::new(std::env::var_os("OUT_DIR").as_ref().unwrap()).join("logo_16x16.png"))
        .unwrap();

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    let image = ico::IconImage::from_rgba_data(16, 16, pixmap.take());
    icon_dir.add_entry(IconDirEntry::encode(&image).unwrap());

    for s in [24, 32, 48, 128, 256] {
        let mut pixmap = Pixmap::new(s, s).unwrap();

        resvg::render(
            &tree,
            resvg::FitTo::Size(s, s),
            Transform::default(),
            pixmap.as_mut(),
        )
        .unwrap();

        let image = ico::IconImage::from_rgba_data(s, s, pixmap.take());
        icon_dir.add_entry(IconDirEntry::encode(&image).unwrap());
    }
    let icon_path = Path::new(std::env::var_os("OUT_DIR").as_ref().unwrap()).join("icon.ico");
    icon_dir.write(File::create(&icon_path).unwrap()).unwrap();

    #[cfg(windows)]
    {
        WindowsResource::new().set_icon(icon_path.to_str().unwrap());
    }
}
