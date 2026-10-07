//! Embedded supplied FreeCAD flat SVG icons. Semantic aliases keep tool call sites stable.
#[cfg(feature = "desktop")]
pub struct Icons;
#[cfg(feature = "desktop")]
impl gpui::AssetSource for Icons {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/add.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-new.svg"
            )),
            "icons/angle_line_to_line.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/append_node.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/arc_3_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Create3PointArc.svg"
            )),
            "icons/arc_center_point_angle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateArc.svg"
            )),
            "icons/arc_concentric.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateArc.svg"
            )),
            "icons/arc_continuation.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateArc.svg"
            )),
            "icons/attributes.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Settings.svg"
            )),
            "icons/bevel.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Chamfer.svg"
            )),
            "icons/black_n_white_mode.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/camera.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/camera-photo.svg"
            )),
            "icons/center_to_page.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/circle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_2_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_2_points_radius.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_3_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Create3PointCircle.svg"
            )),
            "icons/circle_center_point.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_center_radius.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_concentric.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_inscribed.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_tangential_2circles_point.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_tangential_2circles_radius.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_tangential_2points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/circle_tangential_3entities.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/close.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/actions/web-stop.svg"
            )),
            "icons/construction_layer.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_ToggleConstruction.svg"
            )),
            "icons/copy.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-copy.svg"
            )),
            "icons/create_block.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Body.svg"
            )),
            "icons/create_equidistant_polyline.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/create_menu.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/create_polyline_from_existing_segments.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/create_toolbar.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/cursor.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/cut.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-cut.svg"
            )),
            "icons/delete.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-delete.svg"
            )),
            "icons/delete_between_nodes.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-delete.svg"
            )),
            "icons/delete_node.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-delete.svg"
            )),
            "icons/deselect_all.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/deselect_contour.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/deselect_intersected_entities.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/deselect_layer.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/deselect_window.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/dim_aligned.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Length.svg"
            )),
            "icons/dim_angular.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/dim_diametric.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Diameter.svg"
            )),
            "icons/dim_horizontal.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_HorizontalDistance.svg"
            )),
            "icons/dim_leader.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Length.svg"
            )),
            "icons/dim_linear.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Length.svg"
            )),
            "icons/dim_radial.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Radius.svg"
            )),
            "icons/dim_vertical.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_VerticalDistance.svg"
            )),
            "icons/distance_point_to_entity.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Length.svg"
            )),
            "icons/distance_point_to_point.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Length.svg"
            )),
            "icons/divide.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/dockwidgets_bottom.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/dockwidgets_floating.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/dockwidgets_left.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/dockwidgets_right.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-right.svg"
            )),
            "icons/dockwidgets_top.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/down.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-bottom.svg"
            )),
            "icons/downmost.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/draft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Draft.svg"
            )),
            "icons/drawing_settings.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/ellipse_4_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipse_arc_axis.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipse_axis.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipse_center_3_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipse_foci_point.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipse_inscribed.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/ellipses.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/exclusive.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_ValidateSketch.svg"
            )),
            "icons/explode.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/explode_text_to_letters.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/export.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-save.svg"
            )),
            "icons/export_image.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-save.svg"
            )),
            "icons/export_pdf.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-save.svg"
            )),
            "icons/fillet.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Fillet.svg"
            )),
            "icons/fit_to_page.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/grid.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_GridToggle.svg"
            )),
            "icons/hatch.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/import.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-open.svg"
            )),
            "icons/insert_active_block.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Compound.svg"
            )),
            "icons/insert_node.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/invisible.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Measure_Clear_All.svg"
            )),
            "icons/line.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_2p.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_angle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_bisector.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_freehand.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_horizontal.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_parallel.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_parallel_p.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_perpendicular.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_polygon_cen_cor.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_polygon_cor_cor.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_rectangle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateRectangle.svg"
            )),
            "icons/line_relative_angle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_tangent_cc.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_tangent_pc.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_tangent_perpendicular.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/line_vertical.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/lock_rel_zero.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/locked.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/measure.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Measure_Linear.svg"
            )),
            "icons/mirror.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Mirrored.svg"
            )),
            "icons/modify.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mouse.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/move_copy.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_MoveFeature.svg"
            )),
            "icons/move_rotate.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_bottomcenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_bottomleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_bottomright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_middlecenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_middleleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_middleright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_topcenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_topleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/mtext_align_topright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/new.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-new.svg"
            )),
            "icons/new_from_template.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/offset.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Offset2D.svg"
            )),
            "icons/open.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-open.svg"
            )),
            "icons/options.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/paste.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-paste.svg"
            )),
            "icons/points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/polygonal_area.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/polylines.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/polylines_polyline.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/print.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-print.svg"
            )),
            "icons/print_preview.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-print.svg"
            )),
            "icons/properties.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/quit.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/redo.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-redo.svg"
            )),
            "icons/redraw.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/remove.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-delete.svg"
            )),
            "icons/rename_active_block.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/restr_hor.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/restr_ortho.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/restr_ver.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/revert_direction.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/rotate.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-rotate-right.svg"
            )),
            "icons/rotate2.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/save.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-save.svg"
            )),
            "icons/save_as.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/document-save.svg"
            )),
            "icons/scale.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Scaled.svg"
            )),
            "icons/select.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/select_all.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-all.svg"
            )),
            "icons/select_entity.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/select_intersected_entities.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/select_inverted.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/select_window.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/set_rel_zero.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_SelectOrigin.svg"
            )),
            "icons/settings.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/snap_center.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_distance.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_endpoints.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_entity.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_exclusive.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_free.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_grid.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_intersection.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/snap_middle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Snap.svg"
            )),
            "icons/spline.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateBSpline.svg"
            )),
            "icons/spline_points.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateBSpline.svg"
            )),
            "icons/stretch.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_baselinecenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/text_align_baselineleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/text_align_baselineright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/text_align_bottomcenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_bottomleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_bottomright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_middlecenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_middleleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_middleright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_topcenter.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_topleft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/text_align_topright.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/total_length_selected_entities.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-select-box.svg"
            )),
            "icons/trim.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/trim2.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/trim_segments.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/trim_value.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/undo.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/edit-undo.svg"
            )),
            "icons/unlocked.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/up.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-top.svg"
            )),
            "icons/upmost.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/preferences-general.svg"
            )),
            "icons/visible.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Measure_Toggle_All.svg"
            )),
            "icons/zoom_auto.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/zoom-all.svg"
            )),
            "icons/zoom_in.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/actions/web-zoom-in.svg"
            )),
            "icons/zoom_out.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/actions/web-zoom-out.svg"
            )),
            "icons/zoom_pan.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-axonometric.svg"
            )),
            "icons/zoom_previous.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/view-refresh.svg"
            )),
            "icons/zoom_window.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/zoom-all.svg"
            )),
            "icons/Constraint_Block.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Block.svg"
            )),
            "icons/Constraint_EqualLength.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_EqualLength.svg"
            )),
            "icons/Constraint_Horizontal.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Horizontal.svg"
            )),
            "icons/Constraint_Parallel.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Parallel.svg"
            )),
            "icons/Constraint_Perpendicular.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Perpendicular.svg"
            )),
            "icons/Constraint_PointOnObject.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_PointOnObject.svg"
            )),
            "icons/Constraint_PointOnPoint.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_PointOnPoint.svg"
            )),
            "icons/Constraint_Symmetric.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Symmetric.svg"
            )),
            "icons/Constraint_Tangent.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Tangent.svg"
            )),
            "icons/Constraint_Vertical.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Constraint_Vertical.svg"
            )),
            "icons/PartDesign_AdditiveBox.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveBox.svg"
            )),
            "icons/PartDesign_AdditiveCylinder.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveCylinder.svg"
            )),
            "icons/PartDesign_AdditiveHelix.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveHelix.svg"
            )),
            "icons/PartDesign_AdditiveLoft.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveLoft.svg"
            )),
            "icons/PartDesign_AdditivePipe.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditivePipe.svg"
            )),
            "icons/PartDesign_AdditiveSphere.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveSphere.svg"
            )),
            "icons/PartDesign_AdditiveTorus.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_AdditiveTorus.svg"
            )),
            "icons/PartDesign_Boolean.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Boolean.svg"
            )),
            "icons/PartDesign_Chamfer.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Chamfer.svg"
            )),
            "icons/PartDesign_Fillet.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Fillet.svg"
            )),
            "icons/PartDesign_Hole.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Hole.svg"
            )),
            "icons/PartDesign_LinearPattern.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_LinearPattern.svg"
            )),
            "icons/PartDesign_Mirrored.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Mirrored.svg"
            )),
            "icons/PartDesign_MoveFeature.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_MoveFeature.svg"
            )),
            "icons/PartDesign_Pad.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Pad.svg"
            )),
            "icons/PartDesign_PolarPattern.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_PolarPattern.svg"
            )),
            "icons/PartDesign_Revolution.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Revolution.svg"
            )),
            "icons/PartDesign_Scaled.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Scaled.svg"
            )),
            "icons/PartDesign_Thickness.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Thickness.svg"
            )),
            "icons/Part_Compound.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Compound.svg"
            )),
            "icons/Part_Offset2D.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Offset2D.svg"
            )),
            "icons/Sketcher_Create3PointArc.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Create3PointArc.svg"
            )),
            "icons/Sketcher_Create3PointCircle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Create3PointCircle.svg"
            )),
            "icons/Sketcher_CreateArc.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateArc.svg"
            )),
            "icons/Sketcher_CreateBSpline.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateBSpline.svg"
            )),
            "icons/Sketcher_CreateCircle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateCircle.svg"
            )),
            "icons/Sketcher_CreateEllipse.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateEllipse.svg"
            )),
            "icons/Sketcher_CreateFillet.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateFillet.svg"
            )),
            "icons/Sketcher_CreateLine.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateLine.svg"
            )),
            "icons/Sketcher_CreatePoint.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreatePoint.svg"
            )),
            "icons/Sketcher_CreateRectangle.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateRectangle.svg"
            )),
            "icons/Sketcher_CreateRectangle_Center.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateRectangle_Center.svg"
            )),
            "icons/Sketcher_CreateRegularPolygon.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateRegularPolygon.svg"
            )),
            "icons/Sketcher_CreateSlot.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_CreateSlot.svg"
            )),
            "icons/Sketcher_DeleteGeometry.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_DeleteGeometry.svg"
            )),
            "icons/Sketcher_Extend.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Extend.svg"
            )),
            "icons/Sketcher_LeaveSketch.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_LeaveSketch.svg"
            )),
            "icons/Sketcher_NewSketch.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_NewSketch.svg"
            )),
            "icons/Sketcher_RenderingOrder_Construction.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_RenderingOrder_Construction.svg"
            )),
            "icons/Sketcher_Split.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Split.svg"
            )),
            "icons/Sketcher_Trimming.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Sketcher_Trimming.svg"
            )),
            "icons/Group.svg" => Some(include_bytes!("../../assets/icons/freecad flat/Group.svg")),
            "icons/PartDesign_CoordinateSystem.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_CoordinateSystem.svg"
            )),
            "icons/PartDesign_Plane.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/PartDesign_Plane.svg"
            )),
            "icons/Part_CheckGeometry.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_CheckGeometry.svg"
            )),
            "icons/Part_Common.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Common.svg"
            )),
            "icons/Part_CrossSections.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_CrossSections.svg"
            )),
            "icons/Part_Defeaturing.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Defeaturing.svg"
            )),
            "icons/Part_Offset.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Offset.svg"
            )),
            "icons/Part_Section.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Section.svg"
            )),
            "icons/Part_Shapebuilder.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Shapebuilder.svg"
            )),
            "icons/Part_Slice.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Slice.svg"
            )),
            "icons/Part_SliceApart.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_SliceApart.svg"
            )),
            "icons/Part_Thickness.svg" => Some(include_bytes!(
                "../../assets/icons/freecad flat/Part_Thickness.svg"
            )),
            _ => None,
        };
        Ok(bytes.map(std::borrow::Cow::Borrowed))
    }
    fn list(&self, _path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(vec![])
    }
}
