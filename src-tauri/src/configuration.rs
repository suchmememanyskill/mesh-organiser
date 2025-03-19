pub enum SupportedSlicers {
    PrusaSlicer,
    Cura,
    BambuStudio,
    OrcaSlicer,
}

pub struct Configuration {
    pub data_path: String,
    pub prusa_deep_link: bool,
    pub cura_deep_link: bool,
    pub bambu_deep_link: bool,
    pub orca_deep_link: bool,
    pub slicer: SupportedSlicers,
    pub create_popup_on_model_import: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            data_path: String::from(""),
            prusa_deep_link: false,
            cura_deep_link: false,
            bambu_deep_link: false,
            orca_deep_link: false,
            slicer: SupportedSlicers::OrcaSlicer,
            create_popup_on_model_import: false,
        }
    }
}
