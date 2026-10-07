//! Native file pickers run off the GPUI thread. Linux uses the desktop's XDG portal.
#[cfg(feature = "desktop")]
pub fn select_file(
    open: bool,
    suggested: std::path::PathBuf,
) -> std::sync::mpsc::Receiver<Option<std::path::PathBuf>> {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Confusion design", &["con"])
            .set_title(if open { "Open design" } else { "Save design" });
        if open {
            dialog = dialog
                .add_filter("Fusion editable transfer", &["json"])
                .add_filter(
                    "Fusion archive (requires conversion in Fusion)",
                    &["f3d", "f3z"],
                );
        }
        if let Some(parent) = suggested.parent().filter(|p| !p.as_os_str().is_empty()) {
            dialog = dialog.set_directory(parent);
        }
        if !open {
            dialog = dialog.set_file_name(
                suggested
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled.con"),
            );
        }
        let result = if open {
            dialog.pick_file()
        } else {
            dialog.save_file()
        };
        let _ = send.send(result);
    });
    receive
}

/// Printing export picker, with the format selected before opening the dialog.
#[cfg(feature = "desktop")]
pub fn select_export(
    format: crate::exchange::export::ExportFormat,
    suggested: std::path::PathBuf,
) -> std::sync::mpsc::Receiver<Option<std::path::PathBuf>> {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let extension = format.extension();
        let mut suggested = suggested;
        suggested.set_extension(extension);
        let mut dialog = rfd::FileDialog::new()
            .add_filter(extension.to_uppercase(), &[extension])
            .set_title("Export model")
            .set_file_name(
                suggested
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled"),
            );
        if let Some(parent) = suggested.parent().filter(|p| !p.as_os_str().is_empty()) {
            dialog = dialog.set_directory(parent);
        }
        let _ = send.send(dialog.save_file());
    });
    receive
}
