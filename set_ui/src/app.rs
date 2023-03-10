use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::RichText;

pub struct SetUI {
    input: String,
    output: String,

    config: set::SetConfig,
}

impl Default for SetUI {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            config: set::SetConfig {
                variables: true,
                empty_set: true,
                omega: true,
                negated_relations: true,
                subset: true,
                singleton: true,
                comprehension: true,
                power_set: true,
                big_intersection: true,
                big_union: true,
                intersection: true,
                difference: true,
                union: true,
                pair_set: true,
            },
        }
    }
}

impl SetUI {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "m_plus".into(),
            FontData::from_static(include_bytes!("../font/MPLUS.ttf")),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "m_plus".into());
        cc.egui_ctx.set_fonts(fonts);

        SetUI::default()
    }
}

impl eframe::App for SetUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            input,
            output,
            config,
        } = self;
        egui::Window::new("set").collapsible(false).show(ctx, |ui| {
            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Input");
                    ui.add(
                        egui::TextEdit::multiline(input)
                            .desired_width(f32::INFINITY)
                            .desired_rows(1)
                            .lock_focus(true)
                            .clip_text(false),
                    )
                    .request_focus();
                    ui.end_row();
                    ui.label("Eliminate");
                    egui::Grid::new("options_grid").show(ui, |ui| {
                        ui.checkbox(&mut config.negated_relations, "??? ??? ???");
                        ui.checkbox(&mut config.subset, "Subset");
                        ui.checkbox(&mut config.comprehension, "Comprehension");

                        ui.end_row();

                        ui.checkbox(&mut config.big_intersection, "Big intersection");
                        ui.checkbox(&mut config.big_union, "Big union");
                        ui.checkbox(&mut config.power_set, "Power set");

                        ui.end_row();

                        ui.checkbox(&mut config.intersection, "Intersection");
                        ui.checkbox(&mut config.union, "Union");
                        ui.checkbox(&mut config.difference, "Difference");

                        ui.end_row();

                        ui.checkbox(&mut config.singleton, "Singleton");
                        ui.checkbox(&mut config.pair_set, "Pair");
                        ui.checkbox(&mut config.variables, "Variables");

                        ui.end_row();

                        ui.checkbox(&mut config.empty_set, "???");
                        ui.checkbox(&mut config.omega, "??");
                    });
                    ui.end_row();
                    ui.label("Output");
                    ui.add(egui::Label::new(RichText::new(output.clone()).strong()).wrap(true));
                });

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Transform").clicked() {
                    *output = set::run(input, *config);
                }

                egui::warn_if_debug_build(ui);
            });
        });

        egui::Window::new("Symbols")
            .default_open(false)
            .show(ctx, |ui| {
                egui::Grid::new("symbols_grid").show(ui, |ui| {
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("??").clicked() {
                        input.push('??');
                    }
                    if ui.button("??").clicked() {
                        input.push('??');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }

                    ui.end_row();

                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }

                    ui.end_row();

                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }

                    ui.end_row();

                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                    if ui.button("???").clicked() {
                        input.push('???');
                    }
                });
            });
    }
}
