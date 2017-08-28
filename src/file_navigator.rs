#![cfg(windows)]

use nwg::Ui;
use nwg::{Error, FileDialog, FileDialogT};
use nwg::constants::FileDialogAction;
use std::path::PathBuf;

pub struct FileNavigator {}

impl FileNavigator {
    pub fn select_files(extensions: &[&str]) -> Vec<PathBuf> {
        let mut selected: Vec<String> = Vec::new();

        let app: Ui<usize> = Ui::new().expect("Failed to initialize the Ui");

        let ext_filter = if extensions.is_empty() {
            None
        } else {
            let exts = extensions.iter().map(|e| {format!("*.{}", e)}).collect::<Vec<String>>().join(";");
            Some(format!("Needed({})|All(*.*)", exts))
        };

        FileNavigator::setup_ui(&app, ext_filter).unwrap();

        let dialog = match app.get_mut::<FileDialog>(&1) {
            Err(e) => {
                println!("{:?}", e);
                panic!(e)
            }
            Ok(v) => v,
        };

        if dialog.run() {
            let s = &mut dialog.get_selected_items().unwrap();
            selected.clear();
            selected.append(s);
        }

        let selected: Vec<PathBuf> = selected.iter().map(|p| PathBuf::from(p.clone())).collect();
        selected
    }

    fn setup_ui<'a>(ui: &Ui<usize>, filter: Option<String>) -> Result<(), Error> {
        let dialog: FileDialogT<String, usize> = FileDialogT {
            title: String::from("Select something"),
            parent: None,
            action: FileDialogAction::Open,
            multiselect: true,
            default_folder: None,
            filters: filter,
        };

        ui.pack_control(&1, dialog);

        ui.commit()
    }
}
