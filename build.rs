use std::{fs::File, path::Path};

use ico::{IconDir, IconDirEntry};
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree, TreeParsing},
};
#[cfg(windows)]
use winres::WindowsResource;

fn write_icon(s: u32, tree: &Tree, icon_dir: &mut IconDir) {
    let mut pixmap = Pixmap::new(s, s).unwrap();

    resvg::render(
        tree,
        resvg::FitTo::Size(s, s),
        Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    if s == 16 {
        std::fs::write(
            Path::new(std::env::var_os("OUT_DIR").as_ref().unwrap()).join("icon.bitmap"),
            pixmap.data(),
        )
        .unwrap();
    }

    let image = ico::IconImage::from_rgba_data(s, s, pixmap.take());
    icon_dir.add_entry(IconDirEntry::encode(&image).unwrap());
}

fn main() {
    let svg = std::fs::read_to_string("logo.svg").unwrap();
    let tree = Tree::from_str(&svg, &Options::default()).unwrap();

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    {
        let small_svg = std::fs::read_to_string("logo_16.svg").unwrap();
        let small_tree = Tree::from_str(&small_svg, &Options::default()).unwrap();

        write_icon(16, &small_tree, &mut icon_dir)
    }

    for s in [24, 32, 48, 96, 128, 256] {
        write_icon(s, &tree, &mut icon_dir);
    }
    let icon_path = Path::new(std::env::var_os("OUT_DIR").as_ref().unwrap()).join("icon.ico");

    #[cfg(windows)]
    {
        WindowsResource::new().set_icon(icon_path.to_str().unwrap());
    }

    icon_dir.write(File::create(icon_path).unwrap()).unwrap();

    #[cfg(not(windows))]
    println!("cargo:rerun-if-changed=logo.svg")
}
