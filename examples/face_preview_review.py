#!/usr/bin/env python3
"""Exercise desktop planar face selection, live cut previews, signed extrusion, sketch hiding and cancellation in an isolated instrumented copy.

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
os.environ["CARGO_TARGET_DIR"] = str(pathlib.Path(production_metadata["target_directory"]) / "reviews" / "face-preview")
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
            thread_local! {static REGION_REVIEW:std::cell::RefCell<(u8,std::time::Instant)>=std::cell::RefCell::new((0,std::time::Instant::now()));}
            REGION_REVIEW.with(|state| {
                let mut state=state.borrow_mut();assert!(state.1.elapsed().as_secs()<45,"Face review timed out at {}: {:?} / {:?}",state.0,self.error,self.extrusion_preview_error);
                thread_local! {static LAST_STAGE:std::cell::Cell<u8>=const {std::cell::Cell::new(255)};}
                LAST_STAGE.with(|last|if last.replace(state.0)!=state.0{println!("FACE_PREVIEW stage {}: {:?}, {:?}, {:?}",state.0,self.error,self.extrude_operation,self.volume);});
                match state.0 {
                    0=>{
                        let mut design=Design::default();design.rectangle([0.,0.],[0.02,0.02]);
                        self.design=design;self.sketch=true;self.mode=Mode::Sketch;self.rebuild(cx);state.0=1;
                    }
                    1 if self.solved.len()==4=>{
                        self.open_extrude(cx);self.depth.update(cx,|input,cx|input.set_content("10 mm".into(),cx));state.0=2;
                    }
                    2 if self.extrusion_preview_candidate.is_some()&&self.extrusion_preview_revision.is_none()=>{
                        assert!(self.extrusion_preview_error.is_none(),"Stage {}: {:?}",state.0,self.extrusion_preview_error);
                        assert!(self.design.extrusion.is_none());assert!(self.undo.is_empty());
                        self.extrude(cx);assert!(self.pending_candidate.is_some());state.0=3;
                    }
                    3 if self.pending_candidate.is_none()&&self.volume.is_some()=>{
                        assert!((self.volume.unwrap()-0.02*0.02*0.01).abs()<1e-12);
                        assert!(self.hidden_sketches.contains(&self.design.current_sketch_id().unwrap()));
                        self.selected=*self.face_planes.iter().find(|(_,f)|f.nx>0.9).unwrap().0;
                        self.create_sketch(cx);assert!(matches!(self.design.active_plane,SketchPlane::NamedFace{..}));state.0=4;
                    }
                    4 if self.error.is_none()&&self.active_frame().normal.x>0.9&&self.solved.is_empty()=>{
                        let local=self.active_frame().local(nalgebra::Point3::new(0.02,0.01,0.005));
                        self.design.rectangle([local[0]-0.001,local[1]-0.001],[local[0]+0.001,local[1]+0.001]);self.rebuild(cx);state.0=5;
                    }
                    5 if self.solved.len()==4=>{
                        self.open_extrude(cx);assert_eq!(self.extrude_operation,ExtrudeOperation::Cut);
                        self.depth.update(cx,|input,cx|input.set_content("2 mm".into(),cx));state.0=6;
                    }
                    6 if self.extrusion_preview_candidate.is_some()&&self.extrusion_preview_revision.is_none()=>{
                        assert!(self.extrusion_preview_error.is_none(),"Stage {}: {:?}",state.0,self.extrusion_preview_error);
                        assert!(self.design.features.is_empty());assert!((self.volume.unwrap()-0.02*0.02*0.01).abs()<1e-12);
                        self.depth.update(cx,|input,cx|input.set_content("-2 mm".into(),cx));self.extrude_operation=ExtrudeOperation::NewBody;state.0=7;
                    }
                    7 if self.extrusion_preview_key.contains("-2 mm")&&self.extrusion_preview_revision.is_none()=>{
                        assert!(self.extrusion_preview_error.is_none(),"Stage {}: {:?}",state.0,self.extrusion_preview_error);
                        assert!(self.extrusion_preview(cx).unwrap().depth<0.);
                        self.extrude(cx);state.0=8;
                    }
                    8 if self.pending_candidate.is_none()&&!self.design.features.is_empty()=>{
                        assert!(self.hidden_sketches.contains(&self.design.current_sketch_id().unwrap()));
                        assert!((self.volume.unwrap()-(0.02*0.02*0.01+0.002f64.powi(3))).abs()<1e-12);
                        self.open_extrude(cx);self.depth.update(cx,|input,cx|input.set_content("3 mm".into(),cx));state.0=9;
                    }
                    9 if self.extrusion_preview_candidate.is_some()&&self.extrusion_preview_revision.is_none()=>{
                        assert!(self.extrusion_preview_error.is_none());self.panel=None;state.0=10;
                    }
                    10 if self.extrusion_preview_key.is_empty()&&self.error.is_none()&&self.volume.is_some()=>{
                        assert!((self.volume.unwrap()-(0.02*0.02*0.01+0.002f64.powi(3))).abs()<1e-12);
                        self.view_style=crate::render::passes::ViewStyle::ShadedHiddenEdges;self.gpu.set_style(self.view_style);state.0=11;
                    }
                    11=>{
                        self.new_document(cx);self.open_create(crate::model::solid_create::CreateKind::Box,None,cx);self.apply_create(cx);state.0=12;
                    }
                    12 if self.pending_candidate.is_none()&&!self.design.create_features.is_empty()&&self.volume.is_some()=>{
                        assert!(self.error.is_none(),"{:?}",self.error);
                        self.selected=*self.face_planes.iter().find(|(_,f)|f.nz>0.9).unwrap().0;self.create_sketch(cx);
                        assert!(self.error.is_none(),"Create-box face sketch failed: {:?}",self.error);
                        assert!(matches!(self.design.active_plane,SketchPlane::NamedFace{..}));state.0=13;
                    }
                    13 if self.error.is_none()&&(self.active_frame().origin.z-0.02).abs()<1e-9&&self.solved.is_empty()=>{
                        let local=self.active_frame().local(nalgebra::Point3::new(0.01,0.01,0.02));
                        self.design.rectangle([local[0]-0.001,local[1]-0.001],[local[0]+0.001,local[1]+0.001]);self.rebuild(cx);state.0=14;
                    }
                    14 if self.solved.len()==4=>{
                        self.open_extrude(cx);self.depth.update(cx,|input,cx|input.set_content("2 mm".into(),cx));state.0=15;
                    }
                    15 if self.extrusion_preview_candidate.is_some()&&self.extrusion_preview_revision.is_none()=>{
                        assert!(self.extrusion_preview_error.is_none(),"Create-box cut preview: {:?}",self.extrusion_preview_error);self.extrude(cx);state.0=16;
                    }
                    16 if self.pending_candidate.is_none()&&!self.design.features.is_empty()=>{
                        assert!((self.volume.unwrap()-(0.02f64.powi(3)-0.002f64.powi(3))).abs()<1e-12);
                        println!("SKETCH_REGION_REVIEW passed");cx.quit();state.0=17;
                    }
                    _=>{}
                }
            });
'''
    assert text.count(needle) == 1
    path.write_text(text.replace(needle, needle + patch))
    review_manifest = target / "Cargo.toml"
    manifest_text = review_manifest.read_text().replace('edition = "2024"', 'edition = "2024"\nautobins = false', 1)
    review_manifest.write_text(manifest_text + '\n[[bin]]\nname = "confusion-face-preview-review"\npath = "src/main.rs"\n')
    manifest = str(target / "Cargo.toml")
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "desktop", "--bin", "confusion-face-preview-review", "--manifest-path", manifest], cwd=source, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifest], cwd=source))
    binary = root / "confusion-region-review"
    shutil.copy2(pathlib.Path(metadata["target_directory"]) / "release/confusion-face-preview-review", binary)
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env["XDG_CONFIG_HOME"] = str(root / "config")
    env["RUST_BACKTRACE"] = env["RUST_LIB_BACKTRACE"] = "0"
    try:
        run = subprocess.run([str(binary)], env=env, text=True, capture_output=True, timeout=55)
    except subprocess.TimeoutExpired as error:
        print(error.stdout.decode() if isinstance(error.stdout, bytes) else error.stdout)
        print(error.stderr.decode() if isinstance(error.stderr, bytes) else error.stderr)
        raise
    print(run.stdout)
    if run.returncode or "SKETCH_REGION_REVIEW passed" not in run.stdout:
        raise RuntimeError(f"Desktop review exited with {run.returncode}:\n" + run.stderr + run.stdout)
