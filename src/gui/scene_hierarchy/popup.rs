use crate::ecs::{PhysicsProperties, SceneManager};
use crate::gui::gui_state::{GuiState, ScenePanelSelectedItem, SelectedItem};
use crate::gui::scene_hierarchy::predefined_entities::PREDEFINED_ENTITIES;
use crate::gui::scene_hierarchy::utils;
use eframe::egui::{Context, Ui};
use std::path::{Path, PathBuf};
use uuid::Uuid;


pub struct PopupManager {
    pub create_item_name: String,
    pub error_message: String,
    pub rename_input: String,
    pub scene_rename_scene: Option<Uuid>,
    pub entity_rename_entity: Option<(Uuid, Uuid)>,
    pub create_popup_active: bool,
    pub create_item_type: String,
    pub create_entity_popup_active: bool,
    pub create_entity_name: String,
    pub manage_assets_entity: Option<(Uuid, Uuid)>,
    pub manage_assets_popup_active: bool,
    pub resource_selection: Option<(Uuid, Uuid)>,
    pub resource_selection_popup_active: bool,
    pub selected_resource_type: String,
    pub available_resources: Vec<std::path::PathBuf>,
}

impl PopupManager {
    pub fn new() -> Self {
        Self {
            create_item_name: String::new(),
            error_message: String::new(),
            rename_input: String::new(),
            scene_rename_scene: None,
            entity_rename_entity: None,
            create_popup_active: false,
            create_item_type: "Scene".to_string(),
            create_entity_popup_active: false,
            create_entity_name: String::new(),
            manage_assets_entity: None,
            manage_assets_popup_active: false,
            resource_selection: None,
            resource_selection_popup_active: false,
            selected_resource_type: "Images".to_string(),
            available_resources: Vec::new(),
        }
    }

    pub fn start_rename_scene(&mut self, scene_id: Uuid, current_name: String) {
        self.scene_rename_scene = Some(scene_id);
        self.rename_input = current_name;
    }

    pub fn reset_rename_scene(&mut self) {
        self.scene_rename_scene = None;
        self.rename_input.clear();
        self.error_message.clear();
    }

    pub fn start_rename_entity(&mut self, scene_id: Uuid, entity_id: Uuid, current_name: String) {
        self.entity_rename_entity = Some((scene_id, entity_id));
        self.rename_input = current_name;
    }

    pub fn reset_rename(&mut self) {
        self.scene_rename_scene = None;
        self.entity_rename_entity = None;
        self.rename_input.clear();
        self.error_message.clear();
    }

    /// Rename popup, for both scene and entity
    pub fn render_rename_popup(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        gui_state: &mut GuiState,
    ) {

        let (title, scene_id, entity_id) = match self.scene_rename_scene {
            Some(scene_id) => ("Rename Scene", scene_id, None),
            None => match self.entity_rename_entity {
                Some((scene_id, entity_id)) => ("Rename Entity", scene_id, Some(entity_id)),
                None => return,
            },
        };

        // Render rename popup
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label("Enter new name:");
                ui.text_edit_singleline(&mut self.rename_input);

                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() {
                        self.rename_item(scene_id, entity_id, gui_state);
                    }
                    if ui.button("Cancel").clicked() {
                        self.reset_rename();
                    }
                });
            });
    }

    /// Handle renaming
    fn rename_item(&mut self, scene_id: Uuid, entity_id: Option<Uuid>, gui_state: &mut GuiState) {
        if let Some(scene_manager) = gui_state.scene_manager.as_mut() {
            let new_name = self.rename_input.trim().to_string();
            if let Some(entity_id) = entity_id {
                // Rename entity
                if let Some(scene) = scene_manager.get_scene_mut(scene_id) {
                    if let Ok(entity) = scene.get_entity_mut(entity_id) {
                        entity.name = new_name;
                        println!("Renamed entity to: {}", entity.name);
                    }
                }
            } else {
                // Rename scene
                if let Some(scene) = scene_manager.get_scene_mut(scene_id) {
                    scene.name = new_name;
                    println!("Renamed scene to: {}", scene.name);
                }
            }
            utils::save_project(gui_state);
        }
        self.reset_rename();
    }

    /// Render create popup in panel
    pub fn render_create_popup(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        gui_state: &mut GuiState,
    ) {
        egui::Window::new("Create New Node")
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label("Select item to create:");

                ui.vertical(|ui| {
                    let available_width = ui.available_width();

                    let all_item_types = [
                        ("Scene", "Scene"),
                        ("Entity", "Entity"),
                        ("Camera", "Camera"),
                        ("Physics", "Physics"),
                    ];

                    for (type_name, label) in all_item_types {
                        let is_selected = self.create_item_type == type_name;

                        if ui
                            .add_sized(
                                [available_width, 24.0],
                                egui::SelectableLabel::new(is_selected, label),
                            )
                            .clicked()
                        {
                            self.create_item_type = type_name.to_string();
                        }
                    }
                });

                ui.add_space(10.0);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.create_item_name);
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        self.handle_create(gui_state);
                    }

                    if ui.button("Cancel").clicked() {
                        self.reset_create_popup();
                    }
                });

                if !self.error_message.is_empty() {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, &self.error_message);
                }
            });
    }

    fn handle_create(&mut self, gui_state: &mut GuiState) {
        if self.create_item_name.trim().is_empty() {
            self.error_message = "Please select an item type and enter a valid name.".to_string();
            return;
        }

        let item_type = self.create_item_type.clone();
        match item_type.as_str() {
            "Scene" => self.create_new_scene(gui_state),
            "Entity" => self.create_new_entity("Entity".to_string(), gui_state, "Empty"),
            "Camera" => self.create_new_entity("Camera".to_string(), gui_state, "Camera"),
            "Physics" => self.create_new_entity("Physics".to_string(), gui_state, "Physics"),
            other => {
                if PREDEFINED_ENTITIES
                    .iter()
                    .any(|entity| entity.name == other)
                {
                    self.create_new_entity(other.to_string(), gui_state, other);
                }
            }
        }

        if self.error_message.is_empty() {
            self.reset_create_popup();
        }
    }

    fn reset_create_popup(&mut self) {
        self.create_popup_active = false;
        self.create_item_name.clear();
        self.error_message.clear();
    }

    /// Create a new scene
    fn create_new_scene(&mut self, gui_state: &mut GuiState) {
        // Ensure scene manager exists
        let scene_manager = match &mut gui_state.scene_manager {
            Some(manager) => manager,
            None => {
                self.error_message = "Scene manager is not available.".to_string();
                return;
            }
        };

        // Ensure scene name is not empty
        let name = self.create_item_name.trim();
        if name.is_empty() {
            self.error_message = "Scene name cannot be empty.".to_string();
            return;
        }

        // Create the new scene
        match scene_manager.create_scene(name) {
            Ok(new_scene_id) => {
                // Update selection state
                gui_state.scene_panel_selected_item = ScenePanelSelectedItem::Scene(new_scene_id);
                gui_state.selected_item = SelectedItem::Scene(new_scene_id);

                println!("Created new scene '{}' with ID: {:?}", name, new_scene_id);

                // Save the project
                utils::save_project(gui_state);
                self.reset_create_popup();
            }
            Err(e) => {
                self.error_message = format!("Failed to create scene: {}", e);
            }
        }
    }

    /// Create a new entity under the selected scene
    fn create_new_entity(&mut self, entity_type: String, gui_state: &mut GuiState, predefined_type: &str) {
        // Ensure scene manager exists
        let scene_manager = match &mut gui_state.scene_manager {
            Some(manager) => manager,
            None => {
                self.error_message = "Scene manager is not available.".to_string();
                return;
            }
        };

        // Ensure entity name is not empty
        let name = self.create_item_name.trim();
        if name.is_empty() {
            self.error_message = "Entity name cannot be empty.".to_string();
            return;
        }

        let scene_id = match &gui_state.scene_panel_selected_item {
            ScenePanelSelectedItem::Scene(scene_id) => *scene_id,
            _ => {
                self.error_message = "Please select a scene first to add the entity.".to_string();
                return;
            }
        };

        // Get the selected scene
        let scene = match scene_manager.get_scene_mut(scene_id) {
            Some(scene) => scene,
            None => {
                self.error_message = "The selected scene could not be found.".to_string();
                return;
            }
        };

        // Create the new entity
        let new_entity_id = match predefined_type {
            "Empty" => scene.create_entity(name),
            "Camera" => scene.create_camera(name),
            "Physics" => {
                match scene.create_entity(name) {
                    Ok(entity_id) => {
                        if let Ok(entity) = scene.get_entity_mut(entity_id) {
                            if let Some(predefined) = PREDEFINED_ENTITIES.iter().find(|e| e.name == "Physics") {
                                for (attr_name, attr_type, attr_value) in predefined.attributes.iter() {
                                    let _ = entity.create_attribute(attr_name, attr_type.clone(), attr_value.clone());
                                }
                            }
                        }
                        Ok(entity_id)
                    }
                    Err(e) => Err(e)
                }
            },
            _ => scene.create_entity(name),
        };

        match new_entity_id {
            Ok(entity_id) => {
                // Update selection
                gui_state.scene_panel_selected_item = ScenePanelSelectedItem::Entity(scene_id, entity_id);
                gui_state.selected_item = SelectedItem::Entity(scene_id, entity_id);

                println!(
                    "Created new entity '{}' with type '{}' and ID: {:?}",
                    name, entity_type, entity_id
                );

                // Save the project
                utils::save_project(gui_state);
                self.reset_create_popup();
            }
            Err(e) => {
                self.error_message = format!("Failed to create entity: {}", e);
            }
        }
    }

    pub fn render_popups(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, gui_state: &mut GuiState) {
        // Render rename popup
        self.render_rename_popup(ctx, ui, gui_state);

        // Render create popup
        if self.create_popup_active {
            self.render_create_popup(ctx, ui, gui_state);
        }

        // Render manage assets popup
        if self.manage_assets_popup_active {
            if let Some(scene_manager) = &mut gui_state.scene_manager {
                self.show_manage_assets_popup(ctx, scene_manager);
            }
        }

        // Render resource selection popup
        if self.resource_selection.is_some() {
            if let Some(scene_manager) = &mut gui_state.scene_manager {
                self.show_resource_selection_popup(ctx, scene_manager, &gui_state.project_path);
            }
        }
    }

    pub fn show_resource_selection_popup(
        &mut self,
        ctx: &egui::Context,
        scene_manager: &mut SceneManager,
        project_path: &Path,
    ) {
        if let Some((scene_id, entity_id)) = self.resource_selection {
            egui::Window::new("Attach Resource")
                .open(&mut self.resource_selection_popup_active)
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    // Resource type dropdown
                    egui::CollapsingHeader::new("Resource Type")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::ComboBox::from_label("")
                                .selected_text(&self.selected_resource_type)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.selected_resource_type, "Images".to_string(), "Images");
                                    ui.selectable_value(&mut self.selected_resource_type, "Sounds".to_string(), "Sounds");
                                    ui.selectable_value(&mut self.selected_resource_type, "Scripts".to_string(), "Scripts");
                                });
                        });

                    // Update available resources when type changes
                    let resource_path = match self.selected_resource_type.as_str() {
                        "Images" => project_path.join("assets").join("images"),
                        "Sounds" => project_path.join("assets").join("sounds"),
                        "Scripts" => project_path.join("assets").join("scripts"),
                        _ => project_path.join("assets"),
                    };

                    // Read directory and update available resources
                    if let Ok(entries) = std::fs::read_dir(resource_path) {
                        self.available_resources = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.path())
                            .collect();
                    }

                    // Show resource list in a collapsing header
                    egui::CollapsingHeader::new("Available Resources")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for resource_path in &self.available_resources {
                                    if let Some(filename) = resource_path.file_name() {
                                        if ui.button(filename.to_string_lossy().to_string()).clicked() {
                                            if let Some(scene) = scene_manager.get_scene_mut(scene_id) {
                                                if let Ok(entity) = scene.get_entity_mut(entity_id) {
                                                    match self.selected_resource_type.as_str() {
                                                        "Images" => {
                                                            // Check if image is already attached
                                                            if !entity.images.contains(resource_path) {
                                                                entity.images.push(resource_path.clone());
                                                            }
                                                        },
                                                        "Sounds" => {
                                                            // Check if sound is already attached
                                                            if !entity.sounds.contains(resource_path) {
                                                                entity.sounds.push(resource_path.clone());
                                                            }
                                                        },
                                                        "Scripts" => println!("Script selected: {:?}", resource_path),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        });
                });

            // If window is closed, reset the selection
            if !self.resource_selection_popup_active {
                self.resource_selection = None;
            }
        }
    }

    pub fn show_manage_assets_popup(
        &mut self,
        ctx: &egui::Context,
        scene_manager: &mut SceneManager,
    ) {
        if let Some((scene_id, entity_id)) = self.manage_assets_entity {
            egui::Window::new("Manage Assets")
                .open(&mut self.manage_assets_popup_active)
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    if let Some(scene) = scene_manager.get_scene_mut(scene_id) {
                        if let Ok(entity) = scene.get_entity_mut(entity_id) {
                            // Images section as collapsing header
                            if !entity.images.is_empty() {
                                egui::CollapsingHeader::new("Images")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let mut images_to_remove = Vec::new();
                                        for (i, path) in entity.images.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                                                if ui.button("Remove").clicked() {
                                                    images_to_remove.push(i);
                                                }
                                            });
                                        }
                                        // Remove images outside the loop
                                        for &i in images_to_remove.iter().rev() {
                                            entity.images.remove(i);
                                        }
                                    });
                            }

                            // Sounds section as collapsing header
                            if !entity.sounds.is_empty() {
                                egui::CollapsingHeader::new("Sounds")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let mut sounds_to_remove = Vec::new();
                                        for (i, path) in entity.sounds.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                                                if ui.button("Remove").clicked() {
                                                    sounds_to_remove.push(i);
                                                }
                                            });
                                        }
                                        // Remove sounds outside the loop
                                        for &i in sounds_to_remove.iter().rev() {
                                            entity.sounds.remove(i);
                                        }
                                    });
                            }
                        }
                    }
                });
        }
    }

}
