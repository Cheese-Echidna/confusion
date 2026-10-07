#!/usr/bin/env python3
"""Exercise desktop Create/Modify transactions in an isolated instrumented copy.

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
with tempfile.TemporaryDirectory(prefix="confusion-async-apply-") as directory:
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
            thread_local! {
                static APPLY_REVIEW: std::cell::RefCell<(u8, std::time::Instant)> =
                    std::cell::RefCell::new((0, std::time::Instant::now()));
            }
            APPLY_REVIEW.with(|state| {
                let mut state = state.borrow_mut();
                assert!(state.1.elapsed().as_secs() < 20, "Async Apply review timed out at {}", state.0);
                match state.0 {
                    0 => {
                        self.open_create(crate::model::solid_create::CreateKind::Box, None, cx);
                        let start = std::time::Instant::now();
                        self.confirm_editor(window, cx);
                        println!("APPLY_SUBMISSION_MS {}", start.elapsed().as_secs_f64()*1000.);
                        assert!(self.pending_candidate.is_some());
                        assert!(self.design.create_features.is_empty());
                        assert!(!self.dirty(), "Pending candidate marked the document dirty");
                        assert!(self.undo.is_empty());
                        assert!(self.panel == Some(Panel::Create));
                        let revision = self.revision;
                        self.confirm_editor(window, cx);
                        assert_eq!(self.revision, revision, "Duplicate Apply submitted another request");
                        state.0 = 1;
                    }
                    1 if self.pending_candidate.is_none() => {
                        assert_eq!(self.design.create_features.len(), 1);
                        assert!(self.dirty(), "Committed candidate retained cached clean state");
                        assert_eq!(self.undo.len(), 1);
                        assert!(self.panel.is_none());
                        assert!(self.mesh.is_some());
                        self.open_solid_modify(ModifyKind::Scale, cx);
                        self.confirm_editor(window, cx);
                        assert!(self.pending_candidate.is_some());
                        assert!(self.design.solid_edits.is_empty());
                        state.0 = 2;
                    }
                    2 if self.pending_candidate.is_none() => {
                        assert_eq!(self.design.solid_edits.len(), 1);
                        assert_eq!(self.undo.len(), 2);
                        assert!(self.panel.is_none());
                        self.undo_edit(cx);
                        assert!(self.design.solid_edits.is_empty());
                        self.redo_edit(cx);
                        assert_eq!(self.design.solid_edits.len(), 1);
                        state.0 = 3;
                    }
                    3 if self.inspection.is_some() => {
                        self.open_solid_modify(ModifyKind::Fillet, cx);
                        self.solid_editor.fields[0].update(cx, |input,cx| input.set_content("1000 mm".into(),cx));
                        self.confirm_editor(window, cx);
                        assert!(self.pending_candidate.is_some());
                        state.0 = 4;
                    }
                    4 if self.pending_candidate.is_none() => {
                        assert!(self.error.is_some(), "Invalid fillet unexpectedly committed");
                        assert_eq!(self.design.solid_edits.len(), 1);
                        assert_eq!(self.undo.len(), 2);
                        assert!(self.panel == Some(Panel::SolidModify));
                        self.open_create(crate::model::solid_create::CreateKind::Box, None, cx);
                        self.confirm_editor(window,cx);
                        assert!(self.pending_candidate.is_some());
                        self.panel = None; // Same dismissal used by Escape and close.
                        state.0 = 5;
                    }
                    5 if self.pending_candidate.is_none() => {
                        assert_eq!(self.design.create_features.len(), 1);
                        assert_eq!(self.undo.len(), 2);
                        self.open_create(crate::model::solid_create::CreateKind::Box,None,cx);
                        self.confirm_editor(window,cx);
                        self.create_editor.as_ref().unwrap().fields[0].update(cx, |input,cx| input.set_content("15 mm".into(),cx));
                        state.0 = 6;
                    }
                    6 if self.pending_candidate.is_none() => {
                        assert_eq!(self.design.create_features.len(), 1);
                        assert_eq!(self.undo.len(), 2);
                        self.confirm_editor(window,cx);
                        assert!(self.pending_candidate.is_some());
                        self.undo_edit(cx); // Cancel pending Apply without undoing the committed model.
                        assert!(self.pending_candidate.is_none());
                        assert_eq!(self.design.solid_edits.len(), 1);
                        assert_eq!(self.undo.len(), 2);
                        self.confirm_editor(window,cx);
                        assert!(self.pending_candidate.is_some());
                        self.new_document(cx);
                        assert!(self.pending_candidate.is_none());
                        assert!(self.design.create_features.is_empty());
                        state.0 = 7;
                    }
                    7 => {
                        assert!(self.design.create_features.is_empty());
                        assert!(self.undo.is_empty());
                        self.switch_document(0,cx);
                        assert_eq!(self.design.create_features.len(),1);
                        assert_eq!(self.design.solid_edits.len(),1);
                        assert_eq!(self.undo.len(),2);
                        state.0 = 8;
                    }
                    8 if self.inspection.is_some() => {
                        assert!(self.pending_candidate.is_none());
                        assert_eq!(self.evaluated_features.len(),1);
                        println!("ASYNC_APPLY_REVIEW passed");
                        cx.quit();
                        state.0 = 9;
                    }
                    _ => {}
                }
            });
'''
    assert text.count(needle) == 1
    path.write_text(text.replace(needle, needle + patch))
    review_manifest = target / "Cargo.toml"
    manifest_text = review_manifest.read_text().replace('edition = "2024"', 'edition = "2024"\nautobins = false', 1)
    review_manifest.write_text(manifest_text + '\n[[bin]]\nname = "confusion-async-apply-review"\npath = "src/main.rs"\n')
    manifest = str(target / "Cargo.toml")
    subprocess.run(["cargo", "build", "--locked", "--release", "--features", "desktop", "--bin", "confusion-async-apply-review", "--manifest-path", manifest], cwd=source, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifest], cwd=source))
    binary = root / "confusion-apply-review"
    shutil.copy2(pathlib.Path(metadata["target_directory"]) / "release/confusion-async-apply-review", binary)
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env["XDG_CONFIG_HOME"] = str(root / "config")
    env["RUST_BACKTRACE"] = env["RUST_LIB_BACKTRACE"] = "0"
    run = subprocess.run([str(binary)], env=env, text=True, capture_output=True, timeout=30)
    print(run.stdout)
    if run.returncode or "ASYNC_APPLY_REVIEW passed" not in run.stdout:
        raise RuntimeError(f"Desktop review exited with {run.returncode}:\n" + run.stderr + run.stdout)
