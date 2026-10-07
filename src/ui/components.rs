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
        let name = name.to_owned();
        img(move |window: &mut Window, cx: &mut App| {
            use std::{cell::RefCell, collections::HashMap, sync::Arc};
            thread_local! {
                static CACHE: RefCell<HashMap<(String, u32), Arc<RenderImage>>> = RefCell::new(HashMap::new());
            }
            let physical_size = (size * window.scale_factor()).round().max(1.) as u32;
            let key = (name.clone(), physical_size);
            Some(CACHE.with(|cache| {
                if let Some(image) = cache.borrow().get(&key) { return Ok(image.clone()); }
                let path: SharedString = format!("icons/{name}.svg").into();
                let bytes = AssetSource::load(&super::super::assets::Icons, &path)
                    .map_err(|e| gpui::ImageCacheError::Other(Arc::new(e)))?
                    .ok_or_else(|| gpui::ImageCacheError::Asset(format!("Missing icon {name}").into()))?;
                let renderer = cx.svg_renderer();
                let image = rasterize_icon(&renderer, &bytes, physical_size)
                    .map_err(|e| gpui::ImageCacheError::Other(Arc::new(e)))?;
                let mut cache = cache.borrow_mut();
                if cache.len() >= 512 { cache.clear(); }
                cache.insert(key, image.clone());
                Ok(image)
            }))
        })
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
    fn rasterize_icon(
        renderer: &SvgRenderer,
        bytes: &[u8],
        physical_size: u32,
    ) -> gpui::Result<std::sync::Arc<RenderImage>> {
        // GPUI rasterizes SVGs at 2x internally. Keep that modest supersampling
        // instead of shrinking the artwork's intrinsic 512px bitmap into 28px.
        let intrinsic = renderer.render_single_frame(bytes, 1., false)?;
        let scale = 2. * physical_size as f32 / intrinsic.size(0).width.0 as f32;
        Ok(renderer.render_single_frame(bytes, scale, false)?)
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
            .size(px(40.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .cursor_pointer()
            .bg(rgb(if active { t::SELECTED } else { t::PANEL }))
            .hover(|s| s.bg(rgb(t::HOVER)))
            .child(icon(name, 28., if active { t::ACCENT } else { t::TEXT }))
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
    pub fn text_button(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tooltip: &str,
    ) -> Stateful<Div> {
        let tooltip = tooltip.to_owned();
        div()
            .id(id)
            .h(px(30.))
            .px_3()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .cursor_pointer()
            .text_size(px(13.))
            .text_color(rgb(t::TEXT))
            .hover(|style| style.bg(rgb(t::HOVER)))
            .child(label.into())
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
    #[cfg(test)]
    mod tests {
        use super::*;
        #[::core::prelude::v1::test]
        fn every_catalog_icon_loads_and_rasterizes() {
            use crate::ui::toolbar::{Mode, groups};
            let renderer = SvgRenderer::new(std::sync::Arc::new(crate::ui::assets::Icons));
            let mut names = std::collections::HashSet::new();
            for mode in [Mode::Solid, Mode::Sketch, Mode::Drawing] {
                for group in groups(mode) {
                    for feature in group.features {
                        if !names.insert(feature.icon) {
                            continue;
                        }
                        let bytes = AssetSource::load(
                            &crate::ui::assets::Icons,
                            &format!("icons/{}.svg", feature.icon),
                        )
                        .unwrap()
                        .unwrap_or_else(|| panic!("Missing artwork for {}", feature.name));
                        let image = rasterize_icon(&renderer, &bytes, 28).unwrap_or_else(|error| {
                            panic!("Invalid artwork for {}: {error}", feature.name)
                        });
                        assert!(
                            image
                                .as_bytes(0)
                                .unwrap()
                                .as_chunks::<4>()
                                .0
                                .iter()
                                .any(|pixel| pixel[3] > 0),
                            "Empty artwork for {}",
                            feature.name
                        );
                    }
                }
            }
        }
        #[::core::prelude::v1::test]
        fn icons_follow_display_size_and_scale_with_supersampled_alpha() {
            let renderer =
                SvgRenderer::new(std::sync::Arc::new(super::super::super::assets::Icons));
            let diagonal = br#"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256"><path d="M20 30 L220 195" stroke="white" stroke-width="8"/></svg>"#;
            let image = rasterize_icon(&renderer, diagonal, 28).unwrap();
            assert!(
                image
                    .as_bytes(0)
                    .unwrap()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| p[3] > 0 && p[3] < 255)
            );
            for name in ["line", "circle", "down"] {
                let bytes = AssetSource::load(
                    &super::super::super::assets::Icons,
                    &format!("icons/{name}.svg"),
                )
                .unwrap()
                .unwrap();
                for physical_size in [16, 28, 56] {
                    let image = rasterize_icon(&renderer, &bytes, physical_size).unwrap();
                    assert_eq!(image.size(0).width.0, (physical_size * 2) as i32);
                    assert_eq!(image.size(0).height.0, (physical_size * 2) as i32);
                }
            }
        }
    }
}
#[cfg(feature = "desktop")]
pub use implementation::*;
