//! Consistent icon buttons, tooltips and separators for native GPUI chrome.
//! Exports icon/button/separator; callers attach semantic actions. Unimplemented tools
//! remain clickable to explain their status, with a small marker and explicit tooltip.
#[cfg(feature = "desktop")]
mod implementation {
    use super::super::theme as t;
    use gpui::{prelude::*, *};
    struct Tip(String);
    impl Render for Tip {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .px_3()
                .py_2()
                .bg(rgb(t::PANEL))
                .border_1()
                .border_color(rgb(t::BORDER))
                .rounded_sm()
                .text_size(px(12.))
                .text_color(rgb(t::TEXT))
                .child(self.0.clone())
        }
    }
    pub fn icon(name: &str, size: f32, color: u32) -> Img {
        img(ImageSource::Resource(Resource::Embedded(
            SharedString::from(format!("icons/{name}.svg")),
        )))
        .size(px(size))
        .flex_none()
        .opacity(if color == t::DISABLED {
            0.55
        } else if color == t::MUTED {
            0.8
        } else {
            1.
        })
    }
    pub fn button(
        id: impl Into<ElementId>,
        name: &str,
        label: &str,
        active: bool,
        available: bool,
    ) -> Stateful<Div> {
        let tooltip = format!(
            "{label}{}",
            if available { "" } else { " · Not implemented" }
        );
        div()
            .id(id)
            .relative()
            .size(px(32.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .cursor_pointer()
            .bg(rgb(if active { t::SELECTED } else { t::PANEL }))
            .hover(|s| s.bg(rgb(t::HOVER)))
            .child(icon(name, 20., if active { t::ACCENT } else { t::TEXT }))
            .when(!available, |el| {
                el.child(
                    div()
                        .absolute()
                        .right(px(3.))
                        .bottom(px(3.))
                        .size(px(4.))
                        .rounded_full()
                        .bg(rgb(t::DISABLED)),
                )
            })
            .tooltip(move |_, cx| cx.new(|_| Tip(tooltip.clone())).into())
    }
    pub fn separator() -> Div {
        div()
            .w(px(1.))
            .h(px(24.))
            .mx_2()
            .bg(rgb(t::BORDER))
            .flex_none()
    }
}
#[cfg(feature = "desktop")]
pub use implementation::*;
