use egui_winit_vulkano::Gui;
use vulkano::format::Format;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ImagesState {
    pub count: usize,
    pub format: Format,
}

pub struct SwapchainState {
    pub size: [u32; 2],
    pub images_state: ImagesState,
}

pub struct RenderState<'a> {
    swapchain: SwapchainState,
    selected_image: usize,
    gui: &'a mut Gui,
}
