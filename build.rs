use std::path::Path;

use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree, TreeParsing},
};

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

    println!("cargo:rerun-if-changed=logo.svg");
}
