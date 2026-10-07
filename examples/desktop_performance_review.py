#!/usr/bin/env python3
"""Measure desktop frame intervals without GUI automation or production edits.

Generate fixtures with the performance_review Rust example first, then run:
  nix-shell --run 'python3 examples/desktop_performance_review.py'
Requires a desktop display. Instrumented builds use a separate Cargo target
cache under reviews/desktop-performance and copy their executable before running.
No mouse/keyboard input is sent.
"""
import argparse
import json
import os
import pathlib
import shutil
import statistics
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--fixture-dir", type=pathlib.Path, default=pathlib.Path("/tmp/confusion-performance"))
parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("/tmp/confusion-performance/desktop-summary.json"))
parser.add_argument("--font-family", help="Optional isolated font experiment; requires an installed font")
parser.add_argument("--pockets", type=int, choices=[0, 1, 8, 24], nargs="+", default=[0, 1, 8, 24])
parser.add_argument("--backtraces", choices=["0", "1"], nargs="+", default=["0", "1"])
parser.add_argument("--viewport-size", type=int, nargs=2, metavar=("WIDTH", "HEIGHT"), help="Force viewport logical size (physical size is recorded in results)")
args = parser.parse_args()
source = pathlib.Path(__file__).resolve().parents[1]
# Instrumented libraries must also be isolated from the production Cargo cache.
production_metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=source))
os.environ["CARGO_TARGET_DIR"] = str(pathlib.Path(production_metadata["target_directory"]) / "reviews" / "desktop-performance")
temporary = tempfile.TemporaryDirectory(prefix="confusion-desktop-review-")
target = pathlib.Path(temporary.name) / "source"
target.mkdir()
for name in ["Cargo.toml", "Cargo.lock", "build.rs", "src", "assets", "examples", "tests"]:
    origin = source / name
    if origin.is_dir(): shutil.copytree(origin, target / name)
    elif origin.exists(): shutil.copy2(origin, target / name)
p = target / "src/ui/viewport.rs"
s = p.read_text()
a = s.index("pub fn new(surface:")
b = s.index("pub fn focus(", a)
part = s[a:b].replace("            Self {", "            let mut this = Self {", 1)
part = part.replace('''                section_offset: cx.new(|cx| TextInput::new("0 mm", cx)),
            }
        }''','''                section_offset: cx.new(|cx| TextInput::new("0 mm", cx)),
            };
            let path = std::env::var("CONFUSION_REVIEW_DESIGN").expect("review design");
            this.design = crate::persistence::container::load(std::path::Path::new(&path)).expect("review fixture load");
            this.saved_fingerprint = crate::document::dirty::fingerprint(&this.design);
            this.fit_pending = true;
            this.rebuild(cx);
            this
        }''')
assert 'review fixture load' in part
s = s[:a] + part + s[b:]
needle = '''        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
'''
patch = '''            use std::time::Instant;
            thread_local! {
                static REVIEW: std::cell::RefCell<(Instant, Instant, Vec<f64>, Vec<f64>)> = std::cell::RefCell::new((Instant::now(), Instant::now(), vec![], vec![]));
            }
            REVIEW.with(|review| {
                let mut r = review.borrow_mut();
                let now = Instant::now();
                let seconds = now.duration_since(r.0).as_secs_f64();
                let delta = now.duration_since(r.1).as_secs_f64();
                r.1 = now;
                if (3.0..7.0).contains(&seconds) { r.2.push(delta * 1000.); }
                if (7.0..11.0).contains(&seconds) {
                    r.3.push(delta * 1000.);
                    self.camera.orbit([delta * 45., delta * 8.]);
                    self.changed_camera();
                }
                if seconds >= 11.0 {
                    println!("REVIEW_RESULT {}", serde_json::json!({
                        "design": std::env::var("CONFUSION_REVIEW_DESIGN").unwrap(),
                        "backtrace": std::env::var("RUST_LIB_BACKTRACE").unwrap_or_default(),
                        "triangles": self.mesh.as_ref().map(|(_,i)| i.len()/3),
                        "features": self.evaluated_features.len(),
                        "viewport_size": self.gpu.surface.size(),
                        "error": self.error,
                        "idle_ms": r.2,
                        "orbit_ms": r.3,
                    }));
                    cx.quit();
                }
            });
'''
assert s.count(needle) == 1
if args.viewport_size:
    width, height = args.viewport_size
    original = 'wgpu_surface(self.gpu.surface.clone()).absolute().inset_0()'
    assert s.count(original) == 1
    s = s.replace(original, f'wgpu_surface(self.gpu.surface.clone()).absolute().w(px({width}.0)).h(px({height}.0))')
p.write_text(s.replace(needle, needle + patch))

if args.font_family:
    text = p.read_text()
    assert text.count('.font_family(self.ui_font_family.clone())') == 1
    p.write_text(text.replace('.font_family(self.ui_font_family.clone())', '.font_family(' + json.dumps(args.font_family) + ')'))
review_manifest = target / "Cargo.toml"
manifest_text = review_manifest.read_text().replace('edition = "2024"', 'edition = "2024"\nautobins = false', 1)
review_manifest.write_text(manifest_text + '\n[[bin]]\nname = "confusion-desktop-review"\npath = "src/main.rs"\n')
manifest = str(target / "Cargo.toml")
subprocess.run(["cargo", "build", "--locked", "--release", "--features", "desktop", "--bin", "confusion-desktop-review", "--manifest-path", manifest], cwd=source, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifest], cwd=source))
binary = pathlib.Path(temporary.name) / "confusion-review"
shutil.copy2(pathlib.Path(metadata["target_directory"]) / "release" / "confusion-desktop-review", binary)
results = []
for trace in args.backtraces:
    for pockets in args.pockets:
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)  # Match this review's native X11 measurements.
        env["RUST_BACKTRACE"] = env["RUST_LIB_BACKTRACE"] = trace
        env["XDG_CONFIG_HOME"] = str(pathlib.Path(temporary.name) / "config")
        env["CONFUSION_REVIEW_DESIGN"] = str(args.fixture_dir.resolve() / f"pockets-{pockets}.con")
        run = subprocess.run([str(binary)], env=env, text=True, capture_output=True, timeout=30, check=True)
        lines = [line for line in run.stdout.splitlines() if line.startswith("REVIEW_RESULT ")]
        if len(lines) != 1: raise RuntimeError(run.stderr + run.stdout)
        result = json.loads(lines[0][len("REVIEW_RESULT "):])
        result["font_family"] = args.font_family or "resolved system sans"
        if result["error"] or result["features"] != pockets + 1 or result["triangles"] is None:
            raise RuntimeError(f"Fixture did not finish evaluating: {result}")
        for phase in ["idle", "orbit"]:
            times = sorted(result.pop(phase + "_ms"))
            if not times: raise RuntimeError("No frame samples")
            result[phase] = {"fps": 1000 / statistics.mean(times), "median_ms": statistics.median(times), "p95_ms": times[int(len(times) * .95)], "frames": len(times)}
        results.append(result)
        print(json.dumps(result), flush=True)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(results, indent=2) + "\n")
temporary.cleanup()
