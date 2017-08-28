use nwg;
use nwg::{Ui, Event, EventArgs, dispatch_events, exit as nwg_exit};
use nwg::constants::{FONT_WEIGHT_BLACK, FONT_DECO_ITALIC, CheckState, FileDialogAction, HTextAlign, PickerDate};

nwg_template!(
    head: setup_ui<&'static str>,
    controls: [
        ("MainWindow", nwg_window!(title="Nwg Showcase"; position=(100, 100); size=(500, 400))),
        ("FileMenu", nwg_menu!(parent="MainWindow"; text="&File")),
        ("FileDialogButton", nwg_button!(parent="MainWindow"; text="Browse File"; position=(10,120); size=(100, 30); font=Some("Font1"))),
        ("FilePathInput", nwg_textinput!(parent="MainWindow"; position=(120, 125); size=(300, 24); readonly=true; font=Some("Font1"))),
        ("FileDialog", nwg_filedialog!(parent=Some("MainWindow"); action=FileDialogAction::Open; filters=Some("Sheets(*.png;*.json)|Any(*.*)")))
    ];
    events: [
        ("FileDialogButton", "ChooseFile", Event::Click, |app,_,_,_|{
            let (dialog, file_path) = nwg_get_mut!(app; [
                ("FileDialog", nwg::FileDialog),
                ("FilePathInput", nwg::TextInput)
            ]);

            if dialog.run() {
                file_path.set_text(&dialog.get_selected_item().unwrap());
            }
        })
    ];
    resources: [
        ("Font1", nwg_font!(family="Calibri"; size=20 )),
        ("Font2", nwg_font!(family="Arial"; size=17; weight=FONT_WEIGHT_BLACK; decoration=FONT_DECO_ITALIC))
    ];
    values: []
);