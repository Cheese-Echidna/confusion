#!/usr/bin/env python3
"""Exercise desktop sketch-region selection, extrusion and face-reference repair in an isolated instrumented copy.

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
os.environ["CARGO_TARGET_DIR"] = str(pathlib.Path(production_metadata["target_directory"]) / "reviews" / "async-apply")
with tempfile.TemporaryDirectory(prefix="confusion-sketch-region-") as directory:
    root = pathlib.Path(directory)
    target = root / "source"
    target.mkdir()
    for name in ["Cargo.toml", "Cargo.lock", "build.rs", "src", "assets", "examples", "tests"]:
        origin = source / name
        if origin.is_dir(): shutil.copytree(origin, target / name)
        elif origin.exists(): shutil.copy2(origin, target / name)
    path = target / "src/ui/viewport.rs"
    text = path.read_text()
    needle = "        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {\n"
    patch = r'''
            thread_local! {static REGION_REVIEW:std::cell::RefCell<(u8,std::time::Instant)>=std::cell::RefCell::new((0,std::time::Instant::now()));}
            REGION_REVIEW.with(|state| {
                let mut state=state.borrow_mut();assert!(state.1.elapsed().as_secs()<30,"Region review timed out at {}: {:?}",state.0,self.error);
                match state.0 {
                    0 => {
                        let mut design=Design::default();design.rectangle([0.,0.],[0.04,0.02]);design.line([0.02,-0.01],[0.02,0.03]);
                        self.design=design;self.sketch=true;self.mode=Mode::Sketch;self.tool=Tool::Select;self.rebuild(cx);self.fit();state.0=1;
                    }
                    1 if self.solved.len()==self.design.points.len() && self.bounds.get().is_some() => {
                        let bounds=self.bounds.get().unwrap();let click=self.sketch_screen([0.01,0.01]).unwrap();
                        self.sketch_click(bounds.origin+point(px(click[0] as f32),px(click[1] as f32)),false,true,cx);
                        assert!(self.sketch_region.is_some(),"Interior did not select a region");
                        let boundary=self.sketch_region.as_ref().unwrap().boundary.clone();assert!(boundary.iter().any(crate::sketch::regions::is_boundary_token));
                        self.open_extrude(cx);self.depth.update(cx,|input,cx|input.set_content("10 mm".into(),cx));assert!(self.extrusion_preview(cx).is_some());self.extrude(cx);
                        assert!(self.error.is_none(),"Extrude failed: {:?}",self.error);assert_eq!(self.design.extrusion.as_ref().unwrap().boundary,boundary);state.0=2;
                    }
                    2 if self.volume.is_some() => {
                        assert!((self.volume.unwrap()-0.02*0.02*0.01).abs()<1e-10);
                        let face=*self.face_names.iter().find(|(_,key)|key.ends_with("/cap/end")).unwrap().0;self.selected=face;
                        self.open_solid_modify(ModifyKind::PressPull,cx);self.confirm_editor(window,cx);assert!(self.pending_candidate.is_some(),"Apply failed: {:?}",self.error);state.0=3;
                    }
                    3 if self.pending_candidate.is_none() && !self.design.solid_edits.is_empty() => {
                        assert!(self.design.solid_edits[0].face_reference.as_ref().unwrap().ends_with("/cap/end"));
                        self.design.solid_edits[0].face_reference=Some("missing/face".into());self.rebuild(cx);state.0=4;
                    }
                    4 if self.error.is_some() => {
                        assert!(self.error.as_ref().unwrap().contains("reselect"));let id=self.design.solid_edits[0].id;self.open_modify_edit(id,cx);assert!(self.construction_cursor.is_some());state.0=5;
                    }
                    5 if !self.face_names.is_empty() && self.error.is_none() => {
                        self.selected=*self.face_names.iter().find(|(_,key)|key.ends_with("/cap/end")).unwrap().0;
                        self.solid_editor.reference=self.face_names.get(&self.selected).cloned();self.confirm_editor(window,cx);assert!(self.pending_candidate.is_some(),"Repair Apply failed: {:?}",self.error);state.0=6;
                    }
                    6 if self.pending_candidate.is_none() && self.panel.is_none() => {
                        assert!(self.error.is_none());assert_eq!(self.design.solid_edits.len(),1);assert!(self.design.solid_edits[0].face_reference.as_ref().unwrap().ends_with("/cap/end"));
                        assert!((self.volume.unwrap()-0.02*0.02*0.011).abs()<1e-10);
                        self.new_document(cx);let mut design=Design::default();crate::sketch::edit::ellipse(&mut design,[0.,0.],[0.02,0.],[0.,0.01]).unwrap();design.line([0.,-0.03],[0.,0.03]);self.design=design;self.sketch=true;self.mode=Mode::Sketch;self.tool=Tool::Select;self.rebuild(cx);self.fit();state.0=7;
                    }
                    7 if self.solved.len()==self.design.points.len() => {
                        let bounds=self.bounds.get().unwrap();let click=self.sketch_screen([0.01,0.]).unwrap();self.sketch_click(bounds.origin+point(px(click[0] as f32),px(click[1] as f32)),false,true,cx);
                        assert!(self.sketch_region.is_some(),"Ellipse region did not select");self.open_extrude(cx);self.depth.update(cx,|input,cx|input.set_content("10 mm".into(),cx));self.extrude(cx);assert!(self.error.is_none(),"Ellipse Apply failed: {:?}",self.error);state.0=8;
                    }
                    8 if self.volume.is_some() => {
                        assert!((self.volume.unwrap()-std::f64::consts::PI*0.02*0.01*0.01/2.).abs()<1e-10);
                        println!("SKETCH_REGION_REVIEW passed");cx.quit();state.0=9;
                    }
                    _=>{}
                }
            });
'''
    assert text.count(needle) == 1
    path.write_text(text.replace(needle, needle + patch))
    review_manifest = target / "Cargo.toml"
    manifest_text = review_manifest.read_text().replace('edition = "2024"', 'edition = "2024"\nautobins = false', 1)
    review_manifest.write_text(manifest_text + '\n[[bin]]\nname = "confusion-sketch-region-review"\npath = "src/main.rs"\n')
    manifest = str(target / "Cargo.toml")
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "desktop", "--bin", "confusion-sketch-region-review", "--manifest-path", manifest], cwd=source, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifest], cwd=source))
    binary = root / "confusion-region-review"
    shutil.copy2(pathlib.Path(metadata["target_directory"]) / "release/confusion-sketch-region-review", binary)
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env["XDG_CONFIG_HOME"] = str(root / "config")
    env["RUST_BACKTRACE"] = env["RUST_LIB_BACKTRACE"] = "0"
    run = subprocess.run([str(binary)], env=env, text=True, capture_output=True, timeout=45)
    print(run.stdout)
    if run.returncode or "SKETCH_REGION_REVIEW passed" not in run.stdout:
        raise RuntimeError(f"Desktop review exited with {run.returncode}:\n" + run.stderr + run.stdout)
