#!/usr/bin/env python3
"""Exercise desktop extrusion, constraints, topology selection, parameter handles and asynchronous dragging in an isolated instrumented copy.

Run in nix-shell on a graphical desktop. No input events are sent and no user
settings/documents are changed. Assertions run in the real WorkspaceView.
"""
import json
import os
import pathlib
import shutil
import subprocess
import tempfile

source = pathlib.Path(__file__).resolve().parents[1]
# Instrumented libraries must also be isolated from the production Cargo cache.
production_metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=source))
os.environ["CARGO_TARGET_DIR"] = str(pathlib.Path(production_metadata["target_directory"]) / "reviews" / "interaction")
with tempfile.TemporaryDirectory(prefix="confusion-sketch-region-") as directory:
    root = pathlib.Path(directory)
    target = root / "source"
    target.mkdir()
    (target / "vendor").symlink_to(source / "vendor", target_is_directory=True)
    for name in ["Cargo.toml", "Cargo.lock", "build.rs", "src", "assets", "examples", "tests"]:
        origin = source / name
        if origin.is_dir(): shutil.copytree(origin, target / name)
        elif origin.exists(): shutil.copy2(origin, target / name)
    path = target / "src/ui/viewport.rs"
    text = path.read_text()
    needle = "        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {\n"
    patch = r'''
            thread_local! {static REVIEW: std::cell::RefCell<(u8,std::time::Instant)> = std::cell::RefCell::new((0,std::time::Instant::now()));}
            REVIEW.with(|state| {
                let mut state = state.borrow_mut();
                assert!(state.1.elapsed().as_secs() < 45, "Interaction review timed out at {}: {:?} / {:?}", state.0, self.error, self.extrusion_preview_error);
                match state.0 {
                    0 => {
                        self.design = Design::default(); self.sketch = true; self.mode = Mode::Sketch;
                        self.design.point([0.,0.]); self.design.point([0.02,0.01]);
                        self.rebuild(cx); self.fit(); state.0 = 1;
                    }
                    1 if self.solved.len() == 2 && self.bounds.get().is_some() => {
                        self.selection.clear(); self.constrain_selection(Action::Coincident,cx);
                        assert_eq!(self.pending_constraint,Some(Action::Coincident));
                        let bounds = self.bounds.get().unwrap();
                        for at in [[0.,0.],[0.02,0.01]] {
                            let pixel = self.sketch_screen(at).unwrap();
                            self.sketch_click(bounds.origin + point(px(pixel[0] as f32),px(pixel[1] as f32)),false,true,cx);
                        }
                        assert!(self.pending_constraint.is_none());
                        assert_eq!(self.design.constraints.len(),1);
                        assert!(self.error.is_none());
                        self.design = Design::default(); self.design.rectangle([0.,0.],[0.04,0.02]);
                        self.selection.clear(); self.rebuild(cx); self.fit(); state.0 = 2;
                    }
                    2 if self.solved.len() == 4 => {
                        self.open_extrude(cx);
                        assert!(self.editing_feature.is_none());
                        assert!(self.camera.up().dot(&self.active_frame().normal) > 0.9);
                        let preview = self.extrusion_preview(cx).unwrap();
                        let bounds = self.bounds.get().unwrap();
                        let size = [f64::from(bounds.size.width) as u32,f64::from(bounds.size.height) as u32];
                        let a = self.camera.project(nalgebra::Point3::from(preview.frame.world(preview.center).coords*25.),size).unwrap();
                        let b = self.camera.project(nalgebra::Point3::from((preview.frame.world(preview.center)+preview.frame.normal*0.005).coords*25.),size).unwrap();
                        assert!((a[0]-b[0]).abs()<1e-5 && b[1]<a[1]);
                        self.extrude(cx); state.0 = 3;
                    }
                    3 if self.pending_candidate.is_none() && self.volume.is_some() => {
                        assert_eq!(self.design.solid_features().len(),1);
                        self.selected = 0; self.open_extrude(cx);
                        assert!(self.editing_feature.is_none());
                        let candidate = self.extrusion_candidate(cx).unwrap();
                        assert_eq!(candidate.solid_features().len(),2);
                        self.panel = None; self.rebuild(cx); state.0 = 4;
                    }
                    4 if !self.body_edges.is_empty() && self.extrusion_preview_key.is_empty() => {
                        self.open_solid_modify(ModifyKind::Fillet,cx);
                        let bounds = self.bounds.get().unwrap();
                        let size = [f64::from(bounds.size.width) as u32,f64::from(bounds.size.height) as u32];
                        let edges: Vec<_> = self.body_edges.iter().enumerate().filter(|(_,e)| e.points.len()==2 && e.points.iter().all(|p| (p.z-0.005).abs()<1e-8)).take(2).map(|(i,e)| {
                            let a=&e.points[0];let b=&e.points[1];(i,nalgebra::Point3::new((a.x+b.x)*12.5,(a.y+b.y)*12.5,(a.z+b.z)*12.5))
                        }).collect();
                        assert_eq!(edges.len(),2);
                        for (_,at) in edges {
                            let pixel = self.camera.project(at,size).unwrap();
                            assert!(self.pick_topology(bounds.origin+point(px(pixel[0] as f32),px(pixel[1] as f32)),false));
                        }
                        assert_eq!(self.selected_edges.len(),2);
                        assert_eq!(self.parameter_handles(cx).len(),1);
                        let candidate = self.solid_modify_candidate(cx).unwrap();
                        assert_eq!(candidate.solid_edits[0].edge_points.len(),2);
                        self.panel = None; self.rebuild(cx); state.0 = 5;
                    }
                    5 if self.pending_candidate.is_none() && !self.face_planes.is_empty() => {
                        self.selected = *self.face_names.iter().find(|(_,key)| key.ends_with("/cap/end")).unwrap().0;
                        self.open_extrude(cx); assert!(self.extrusion_face.is_some());
                        assert_eq!(self.extrude_operation,ExtrudeOperation::Join);
                        let candidate = self.extrusion_candidate(cx).unwrap();
                        assert_eq!(candidate.solid_edits.len(),1);
                        assert!(self.extrusion_preview(cx).is_some());
                        self.panel = Some(Panel::Inspect); self.inspection_action = Action::Measure;
                        self.extrusion_face = None; self.selected = 0;
                        let bounds = self.bounds.get().unwrap();
                        let size = [f64::from(bounds.size.width) as u32,f64::from(bounds.size.height) as u32];
                        let corners: Vec<_> = self.body_vertices.iter().take(2).map(|v| nalgebra::Point3::new(v.point.x*25.,v.point.y*25.,v.point.z*25.)).collect();
                        for (i,at) in corners.into_iter().enumerate() {
                            let pixel = self.camera.project(at,size).unwrap();
                            assert!(self.pick_topology(bounds.origin+point(px(pixel[0] as f32),px(pixel[1] as f32)), i>0));
                        }
                        assert_eq!(self.selected_vertices.len(),2);
                        assert!(self.inspection_lines()[0].starts_with("Distance:"));
                        self.panel = None;
                        self.open_create(crate::model::solid_create::CreateKind::Revolve,None,cx);
                        assert_eq!(self.parameter_handles(cx).len(),self.create_editor.as_ref().unwrap().fields.len());
                        self.panel = None; self.create_editor = None;
                        self.sketch = true; self.mode = Mode::Sketch;
                        let mut before = self.display_design(); before.constraints.clear();
                        self.design = before.clone();
                        let id = before.points[0].id; let start = before.points[0].xy;
                        self.sketch_drag = Some(SketchDrag { before, points:vec![id], start, moved:false });
                        self.drag_target = Some([start[0]+0.003,start[1]+0.002]); self.poll_drag_solve(cx);
                        assert!(self.drag_solve.is_some());
                        self.drag_target = Some([start[0]+0.005,start[1]+0.003]);
                        self.finish_sketch_drag(false,cx);
                        assert!(self.drag_released); state.0 = 6;
                    }
                    6 if self.drag_solve.is_none() && self.sketch_drag.is_none() => {
                        assert!(!self.drag_released);
                        assert!((self.design.points[0].xy[0]-0.005).abs()<1e-5);
                        assert!((self.design.points[0].xy[1]-0.003).abs()<1e-5);
                        println!("INTERACTION_REVIEW passed: new extrusion, screen-up axis, tool-first constraints, sequential body edges, parameter tabs, face extrusion, asynchronous drag release");
                        cx.quit(); state.0 = 7;
                    }
                    _ => {}
                }
            });
'''
    assert text.count(needle) == 1
    path.write_text(text.replace(needle, needle + patch))
    review_manifest = target / "Cargo.toml"
    manifest_text = review_manifest.read_text().replace('edition = "2024"', 'edition = "2024"\nautobins = false', 1)
    review_manifest.write_text(manifest_text + '\n[[bin]]\nname = "confusion-interaction-review"\npath = "src/main.rs"\n')
    manifest = str(target / "Cargo.toml")
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "desktop", "--bin", "confusion-interaction-review", "--manifest-path", manifest], cwd=source, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifest], cwd=source))
    binary = root / "confusion-region-review"
    shutil.copy2(pathlib.Path(metadata["target_directory"]) / "release/confusion-interaction-review", binary)
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env["XDG_CONFIG_HOME"] = str(root / "config")
    env["RUST_BACKTRACE"] = env["RUST_LIB_BACKTRACE"] = "0"
    run = subprocess.run([str(binary)], env=env, text=True, capture_output=True, timeout=45)
    print(run.stdout)
    if run.returncode or "INTERACTION_REVIEW passed" not in run.stdout:
        raise RuntimeError(f"Desktop review exited with {run.returncode}:\n" + run.stderr + run.stdout)
