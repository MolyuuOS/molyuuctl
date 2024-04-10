#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Protocol {
    X11,
    Wayland,
}