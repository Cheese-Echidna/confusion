// Exact STEP export coordination, included in viewport's desktop module.
impl WorkspaceView {
    fn request_step_export(&mut self, cx: &mut Context<Self>) {
        if self.step_export_dialog.is_some()
            || self.export_dialog.is_some()
            || self.export_result.is_some()
        {
            return;
        }
        let mut suggested = self
            .saved_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("Untitled.con"));
        suggested.set_extension("step");
        let (send, receive) = std::sync::mpsc::channel();
        self.step_export_dialog = Some(receive);
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("STEP solid model", &["step", "stp"])
                .set_title("Export STEP model")
                .set_file_name(
                    suggested
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled.step"),
                );
            if let Some(parent) = suggested.parent().filter(|p| !p.as_os_str().is_empty()) {
                dialog = dialog.set_directory(parent);
            }
            let _ = send.send(dialog.save_file());
        });
        cx.notify();
    }

    fn poll_step_export(&mut self) {
        let chosen =
            self.step_export_dialog
                .as_ref()
                .and_then(|receiver| match receiver.try_recv() {
                    Ok(path) => Some(path),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
                    Err(_) => None,
                });
        let Some(path) = chosen else {
            return;
        };
        self.step_export_dialog = None;
        let Some(mut path) = path else {
            return;
        };
        if !path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("step") || s.eq_ignore_ascii_case("stp"))
        {
            path.set_extension("step");
        }
        let design = self.design.clone();
        let (send, receive) = std::sync::mpsc::channel();
        self.export_result = Some(receive);
        self.status = "Exporting exact STEP model…".into();
        std::thread::spawn(move || {
            let result = crate::exchange::step::export_design(&path, &design).map(|_| path);
            let _ = send.send(result);
        });
    }
}
