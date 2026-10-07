//! Embedded LibreCAD CC0 SVG geometry, independent of the launch directory.
//! Exports Icons; application/bootstrap installs this GPUI AssetSource. See the supplied
//! librecad_svg_icons/readme.md for artwork provenance. Existing color slots are mapped
//! to Gruvbox at presentation time; source files and drawing geometry stay unchanged.
#[cfg(feature = "desktop")]
pub struct Icons;
#[cfg(feature = "desktop")]
impl gpui::AssetSource for Icons {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        use std::borrow::Cow;
        let original: Option<Cow<'static, [u8]>> = match path {
            "icons/add.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/add.svg"
            ))),
            "icons/angle_line_to_line.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/angle_line_to_line.svg"
            ))),
            "icons/append_node.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/append_node.svg"
            ))),
            "icons/arc_3_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/arc_3_points.svg"
            ))),
            "icons/arc_center_point_angle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/arc_center_point_angle.svg"
            ))),
            "icons/arc_concentric.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/arc_concentric.svg"
            ))),
            "icons/arc_continuation.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/arc_continuation.svg"
            ))),
            "icons/attributes.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/attributes.svg"
            ))),
            "icons/bevel.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/bevel.svg"
            ))),
            "icons/black_n_white_mode.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/black_n_white_mode.svg"
            ))),
            "icons/camera.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/camera.svg"
            ))),
            "icons/center_to_page.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/center_to_page.svg"
            ))),
            "icons/circle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle.svg"
            ))),
            "icons/circle_2_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_2_points.svg"
            ))),
            "icons/circle_2_points_radius.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_2_points_radius.svg"
            ))),
            "icons/circle_3_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_3_points.svg"
            ))),
            "icons/circle_center_point.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_center_point.svg"
            ))),
            "icons/circle_center_radius.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_center_radius.svg"
            ))),
            "icons/circle_concentric.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_concentric.svg"
            ))),
            "icons/circle_inscribed.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_inscribed.svg"
            ))),
            "icons/circle_tangential_2circles_point.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_tangential_2circles_point.svg"
            ))),
            "icons/circle_tangential_2circles_radius.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_tangential_2circles_radius.svg"
            ))),
            "icons/circle_tangential_2points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_tangential_2points.svg"
            ))),
            "icons/circle_tangential_3entities.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/circle_tangential_3entities.svg"
            ))),
            "icons/close.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/close.svg"
            ))),
            "icons/construction_layer.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/construction_layer.svg"
            ))),
            "icons/copy.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/copy.svg"
            ))),
            "icons/create_block.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/create_block.svg"
            ))),
            "icons/create_equidistant_polyline.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/create_equidistant_polyline.svg"
            ))),
            "icons/create_menu.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/create_menu.svg"
            ))),
            "icons/create_polyline_from_existing_segments.svg" => {
                Some(Cow::Borrowed(include_bytes!(
                    "../../librecad_svg_icons/create_polyline_from_existing_segments.svg"
                )))
            }
            "icons/create_toolbar.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/create_toolbar.svg"
            ))),
            "icons/cursor.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/cursor.svg"
            ))),
            "icons/cut.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/cut.svg"
            ))),
            "icons/delete.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/delete.svg"
            ))),
            "icons/delete_between_nodes.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/delete_between_nodes.svg"
            ))),
            "icons/delete_node.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/delete_node.svg"
            ))),
            "icons/deselect_all.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/deselect_all.svg"
            ))),
            "icons/deselect_contour.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/deselect_contour.svg"
            ))),
            "icons/deselect_intersected_entities.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/deselect_intersected_entities.svg"
            ))),
            "icons/deselect_layer.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/deselect_layer.svg"
            ))),
            "icons/deselect_window.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/deselect_window.svg"
            ))),
            "icons/dim_aligned.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_aligned.svg"
            ))),
            "icons/dim_angular.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_angular.svg"
            ))),
            "icons/dim_diametric.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_diametric.svg"
            ))),
            "icons/dim_horizontal.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_horizontal.svg"
            ))),
            "icons/dim_leader.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_leader.svg"
            ))),
            "icons/dim_linear.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_linear.svg"
            ))),
            "icons/dim_radial.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_radial.svg"
            ))),
            "icons/dim_vertical.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dim_vertical.svg"
            ))),
            "icons/distance_point_to_entity.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/distance_point_to_entity.svg"
            ))),
            "icons/distance_point_to_point.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/distance_point_to_point.svg"
            ))),
            "icons/divide.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/divide.svg"
            ))),
            "icons/dockwidgets_bottom.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dockwidgets_bottom.svg"
            ))),
            "icons/dockwidgets_floating.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dockwidgets_floating.svg"
            ))),
            "icons/dockwidgets_left.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dockwidgets_left.svg"
            ))),
            "icons/dockwidgets_right.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dockwidgets_right.svg"
            ))),
            "icons/dockwidgets_top.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/dockwidgets_top.svg"
            ))),
            "icons/down.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/down.svg"
            ))),
            "icons/downmost.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/downmost.svg"
            ))),
            "icons/draft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/draft.svg"
            ))),
            "icons/drawing_settings.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/drawing_settings.svg"
            ))),
            "icons/ellipse_4_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_4_points.svg"
            ))),
            "icons/ellipse_arc_axis.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_arc_axis.svg"
            ))),
            "icons/ellipse_axis.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_axis.svg"
            ))),
            "icons/ellipse_center_3_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_center_3_points.svg"
            ))),
            "icons/ellipse_foci_point.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_foci_point.svg"
            ))),
            "icons/ellipse_inscribed.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipse_inscribed.svg"
            ))),
            "icons/ellipses.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/ellipses.svg"
            ))),
            "icons/exclusive.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/exclusive.svg"
            ))),
            "icons/explode.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/explode.svg"
            ))),
            "icons/explode_text_to_letters.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/explode_text_to_letters.svg"
            ))),
            "icons/export.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/export.svg"
            ))),
            "icons/export_image.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/export_image.svg"
            ))),
            "icons/export_pdf.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/export_pdf.svg"
            ))),
            "icons/fillet.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/fillet.svg"
            ))),
            "icons/fit_to_page.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/fit_to_page.svg"
            ))),
            "icons/grid.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/grid.svg"
            ))),
            "icons/hatch.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/hatch.svg"
            ))),
            "icons/import.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/import.svg"
            ))),
            "icons/insert_active_block.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/insert_active_block.svg"
            ))),
            "icons/insert_node.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/insert_node.svg"
            ))),
            "icons/invisible.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/invisible.svg"
            ))),
            "icons/line.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line.svg"
            ))),
            "icons/line_2p.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_2p.svg"
            ))),
            "icons/line_angle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_angle.svg"
            ))),
            "icons/line_bisector.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_bisector.svg"
            ))),
            "icons/line_freehand.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_freehand.svg"
            ))),
            "icons/line_horizontal.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_horizontal.svg"
            ))),
            "icons/line_parallel.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_parallel.svg"
            ))),
            "icons/line_parallel_p.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_parallel_p.svg"
            ))),
            "icons/line_perpendicular.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_perpendicular.svg"
            ))),
            "icons/line_polygon_cen_cor.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_polygon_cen_cor.svg"
            ))),
            "icons/line_polygon_cor_cor.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_polygon_cor_cor.svg"
            ))),
            "icons/line_rectangle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_rectangle.svg"
            ))),
            "icons/line_relative_angle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_relative_angle.svg"
            ))),
            "icons/line_tangent_cc.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_tangent_cc.svg"
            ))),
            "icons/line_tangent_pc.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_tangent_pc.svg"
            ))),
            "icons/line_tangent_perpendicular.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_tangent_perpendicular.svg"
            ))),
            "icons/line_vertical.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/line_vertical.svg"
            ))),
            "icons/lock_rel_zero.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/lock_rel_zero.svg"
            ))),
            "icons/locked.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/locked.svg"
            ))),
            "icons/measure.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/measure.svg"
            ))),
            "icons/mirror.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mirror.svg"
            ))),
            "icons/modify.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/modify.svg"
            ))),
            "icons/mouse.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mouse.svg"
            ))),
            "icons/move_copy.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/move_copy.svg"
            ))),
            "icons/move_rotate.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/move_rotate.svg"
            ))),
            "icons/mtext_align_bottomcenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_bottomcenter.svg"
            ))),
            "icons/mtext_align_bottomleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_bottomleft.svg"
            ))),
            "icons/mtext_align_bottomright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_bottomright.svg"
            ))),
            "icons/mtext_align_middlecenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_middlecenter.svg"
            ))),
            "icons/mtext_align_middleleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_middleleft.svg"
            ))),
            "icons/mtext_align_middleright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_middleright.svg"
            ))),
            "icons/mtext_align_topcenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_topcenter.svg"
            ))),
            "icons/mtext_align_topleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_topleft.svg"
            ))),
            "icons/mtext_align_topright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/mtext_align_topright.svg"
            ))),
            "icons/new.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/new.svg"
            ))),
            "icons/new_from_template.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/new_from_template.svg"
            ))),
            "icons/offset.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/offset.svg"
            ))),
            "icons/open.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/open.svg"
            ))),
            "icons/options.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/options.svg"
            ))),
            "icons/paste.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/paste.svg"
            ))),
            "icons/points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/points.svg"
            ))),
            "icons/polygonal_area.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/polygonal_area.svg"
            ))),
            "icons/polylines.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/polylines.svg"
            ))),
            "icons/polylines_polyline.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/polylines_polyline.svg"
            ))),
            "icons/print.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/print.svg"
            ))),
            "icons/print_preview.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/print_preview.svg"
            ))),
            "icons/properties.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/properties.svg"
            ))),
            "icons/quit.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/quit.svg"
            ))),
            "icons/redo.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/redo.svg"
            ))),
            "icons/redraw.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/redraw.svg"
            ))),
            "icons/remove.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/remove.svg"
            ))),
            "icons/rename_active_block.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/rename_active_block.svg"
            ))),
            "icons/restr_hor.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/restr_hor.svg"
            ))),
            "icons/restr_ortho.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/restr_ortho.svg"
            ))),
            "icons/restr_ver.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/restr_ver.svg"
            ))),
            "icons/revert_direction.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/revert_direction.svg"
            ))),
            "icons/rotate.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/rotate.svg"
            ))),
            "icons/rotate2.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/rotate2.svg"
            ))),
            "icons/save.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/save.svg"
            ))),
            "icons/save_as.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/save_as.svg"
            ))),
            "icons/scale.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/scale.svg"
            ))),
            "icons/select.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select.svg"
            ))),
            "icons/select_all.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select_all.svg"
            ))),
            "icons/select_entity.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select_entity.svg"
            ))),
            "icons/select_intersected_entities.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select_intersected_entities.svg"
            ))),
            "icons/select_inverted.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select_inverted.svg"
            ))),
            "icons/select_window.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/select_window.svg"
            ))),
            "icons/set_rel_zero.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/set_rel_zero.svg"
            ))),
            "icons/settings.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/settings.svg"
            ))),
            "icons/snap_center.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_center.svg"
            ))),
            "icons/snap_distance.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_distance.svg"
            ))),
            "icons/snap_endpoints.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_endpoints.svg"
            ))),
            "icons/snap_entity.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_entity.svg"
            ))),
            "icons/snap_exclusive.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_exclusive.svg"
            ))),
            "icons/snap_free.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_free.svg"
            ))),
            "icons/snap_grid.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_grid.svg"
            ))),
            "icons/snap_intersection.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_intersection.svg"
            ))),
            "icons/snap_middle.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/snap_middle.svg"
            ))),
            "icons/spline.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/spline.svg"
            ))),
            "icons/spline_points.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/spline_points.svg"
            ))),
            "icons/stretch.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/stretch.svg"
            ))),
            "icons/text.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text.svg"
            ))),
            "icons/text_align_baselinecenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_baselinecenter.svg"
            ))),
            "icons/text_align_baselineleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_baselineleft.svg"
            ))),
            "icons/text_align_baselineright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_baselineright.svg"
            ))),
            "icons/text_align_bottomcenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_bottomcenter.svg"
            ))),
            "icons/text_align_bottomleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_bottomleft.svg"
            ))),
            "icons/text_align_bottomright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_bottomright.svg"
            ))),
            "icons/text_align_middlecenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_middlecenter.svg"
            ))),
            "icons/text_align_middleleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_middleleft.svg"
            ))),
            "icons/text_align_middleright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_middleright.svg"
            ))),
            "icons/text_align_topcenter.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_topcenter.svg"
            ))),
            "icons/text_align_topleft.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_topleft.svg"
            ))),
            "icons/text_align_topright.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/text_align_topright.svg"
            ))),
            "icons/total_length_selected_entities.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/total_length_selected_entities.svg"
            ))),
            "icons/trim.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/trim.svg"
            ))),
            "icons/trim2.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/trim2.svg"
            ))),
            "icons/trim_segments.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/trim_segments.svg"
            ))),
            "icons/trim_value.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/trim_value.svg"
            ))),
            "icons/undo.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/undo.svg"
            ))),
            "icons/unlocked.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/unlocked.svg"
            ))),
            "icons/up.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/up.svg"
            ))),
            "icons/upmost.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/upmost.svg"
            ))),
            "icons/visible.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/visible.svg"
            ))),
            "icons/zoom_auto.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_auto.svg"
            ))),
            "icons/zoom_in.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_in.svg"
            ))),
            "icons/zoom_out.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_out.svg"
            ))),
            "icons/zoom_pan.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_pan.svg"
            ))),
            "icons/zoom_previous.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_previous.svg"
            ))),
            "icons/zoom_window.svg" => Some(Cow::Borrowed(include_bytes!(
                "../../librecad_svg_icons/zoom_window.svg"
            ))),
            _ => None,
        };
        Ok(original.map(|bytes| Cow::Owned(palette_svg(&bytes))))
    }
    fn list(&self, _path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(vec![])
    }
}

#[cfg(feature = "desktop")]
fn palette_svg(bytes: &[u8]) -> Vec<u8> {
    use super::theme as t;
    let source = String::from_utf8_lossy(bytes);
    let mut output = String::new();
    let mut remaining = source.as_ref();
    while let Some(index) = remaining.find('#') {
        output.push_str(&remaining[..index]);
        remaining = &remaining[index + 1..];
        let count = remaining.bytes().take_while(u8::is_ascii_hexdigit).count();
        if count == 6 || count == 8 || count == 3 {
            let hex = &remaining[..count];
            let rgb = if count == 3 {
                hex.chars().flat_map(|c| [c, c]).collect::<String>()
            } else {
                hex[..6].to_owned()
            };
            let value = u32::from_str_radix(&rgb, 16).unwrap();
            let r = (value >> 16) & 255;
            let g = (value >> 8) & 255;
            let b = value & 255;
            let color = if r.max(g).max(b) - r.min(g).min(b) < 35 {
                if r < 70 {
                    t::TEXT
                } else if r > 195 {
                    t::VIEWPORT
                } else {
                    t::MUTED
                }
            } else if r > g + b / 2 {
                t::WARNING
            } else {
                t::ACCENT
            };
            output.push_str(&format!("#{color:06x}"));
            if count == 8 {
                output.push_str(&hex[6..]);
            }
            remaining = &remaining[count..];
        } else {
            output.push('#');
        }
    }
    output.push_str(remaining);
    output.into_bytes()
}
