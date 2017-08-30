#[cfg(windows)]
pub mod navigator {
    use nwg::Ui;
    use nwg::{Error, FileDialog, FileDialogT};
    use nwg::constants::FileDialogAction;
    use std::path::PathBuf;

    pub struct FileNavigator {}

    impl FileNavigator {
        pub fn select_files() -> Vec<PathBuf> {
            let mut selected: Vec<String> = Vec::new();

            let app: Ui<usize> = Ui::new().expect("Failed to initialize the Ui");

            let ext_filter = format!("Needed(*.png;*.json)|All(*.*)", exts);

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

            let selected: Vec<PathBuf> =
                selected.iter().map(|p| PathBuf::from(p.clone())).collect();
            selected
        }

        fn setup_ui<'a>(ui: &Ui<usize>, filter: String) -> Result<(), Error> {
            let dialog: FileDialogT<String, usize> = FileDialogT {
                title: String::from("Select something"),
                parent: None,
                action: FileDialogAction::Open,
                multiselect: true,
                default_folder: None,
                filters: Some(filter),
            };

            ui.pack_control(&1, dialog);

            ui.commit()
        }
    }
}

#[cfg(unix)]
pub mod navigator {
    extern crate gtk;
    use gtk::*;
    use std::path::PathBuf;

    pub struct FileNavigator;

    impl FileNavigator {
        pub fn select_files() -> Vec<PathBuf> {
            if gtk::init().is_err() {
                println!("Failed to GTK!");
                panic!();
            }

            // let window = Window::new(WindowType::Toplevel);
            // window.set_title("Select shit!");
            // window.set_default_size(50, 50);

            let chooser = FileChooserDialog::new(
                Some("Select sprite sheet with json file, ayy"),
                None::<&Window>,
                FileChooserAction::Open,
            );

            let ff = FileFilter::new();
            FileFilter::add_pattern(&ff, "*.png");
            FileFilter::add_pattern(&ff, "*.json");
            FileFilter::set_name(&ff, "spritesheet");

            let ff2 = FileFilter::new();
            FileFilter::add_pattern(&ff2, "*.*");
            FileFilter::set_name(&ff2, "All");

            chooser.add_filter(&ff);
            chooser.add_filter(&ff2);
            chooser.set_select_multiple(true);

            chooser.add_buttons(&[
                ("Open", ResponseType::Ok.into()),
                ("Cancel", ResponseType::Cancel.into()),
            ]);

            chooser.run();
            let files = chooser.get_filenames();
            chooser.destroy();

            files
        }
    }


}
