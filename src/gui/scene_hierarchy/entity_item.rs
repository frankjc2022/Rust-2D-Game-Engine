use crate::gui::scene_hierarchy::{SceneHierarchy, resource_item::ResourceItem};
use crate::gui::gui_state::{GuiState, ScenePanelSelectedItem, SelectedItem};
use egui::{Context, Ui};
use uuid::Uuid;
use std::collections::HashMap;

pub struct EntityItem;

impl EntityItem {
    pub fn show_entities(
        ui: &mut Ui,
        ctx: &Context,
        hierarchy: &mut SceneHierarchy,
        gui_state: &mut GuiState,
        scene_id: &Uuid,
        entities: &HashMap<Uuid, crate::ecs::Entity>,
    ) {

        // Sort entity by name for displaying
        let mut sorted_entities: Vec<(&Uuid, &crate::ecs::Entity)> = entities.iter().collect();
        sorted_entities.sort_by(|(_, entity_a), (_, entity_b)| {
            entity_a.name.to_lowercase().cmp(&entity_b.name.to_lowercase())
        });

        for (entity_id, entity) in sorted_entities {

            // Filter by search_query for displaying
            if !hierarchy.search_query.is_empty()
                && !entity.name.to_lowercase().contains(&hierarchy.search_query.to_lowercase()) { continue; }

            let header_id = ui.make_persistent_id(entity_id);

            // Show as collapsable if has resources, otherwise show as label.
            if !entity.resource_list.is_empty() {
                egui::collapsing_header::CollapsingState::load_with_default_open(ctx, header_id, true)
                    .show_header(ui, |ui| {
                        EntityItem::tree_item_entity(ui, scene_id, entity_id, &entity.name, hierarchy, gui_state);
                    })
                    .body(|ui| {
                        ResourceItem::show_resources(ui, scene_id, entity_id, &entity.resource_list, hierarchy, gui_state);
                    });
            } else {
                ui.horizontal(|ui| {
                    EntityItem::tree_item_entity(ui, scene_id, entity_id, &entity.name, hierarchy, gui_state);
                });
            }
        }
    }

    fn tree_item_entity(ui: &mut Ui, scene_id: &Uuid, entity_id: &Uuid, entity_name: &str, hierarchy: &mut SceneHierarchy, gui_state: &mut GuiState) {
        let selected = matches!(
            gui_state.scene_panel_selected_item,
            ScenePanelSelectedItem::Entity(s_id, e_id) if s_id == *scene_id && e_id == *entity_id
        );

        let response = ui.selectable_label(selected, format!("📌 {}", entity_name));
        if response.clicked() {
            gui_state.selected_item = SelectedItem::Entity(*scene_id, *entity_id);
            gui_state.scene_panel_selected_item = ScenePanelSelectedItem::Entity(*scene_id, *entity_id);
        }

        response.context_menu(|ui| {
            if ui.button("Manage Resources").clicked() {
                hierarchy.popup_manager.manage_resources_entity = Some((*scene_id, *entity_id));
                hierarchy.popup_manager.manage_resource_popup_active = true;
                ui.close_menu();
            }
            if ui.button("Rename").clicked() {
                hierarchy.popup_manager.entity_rename_entity = Some((*scene_id, *entity_id));
                hierarchy.popup_manager.rename_input = entity_name.to_string();
                hierarchy.popup_manager.start_rename_entity(*scene_id, *entity_id, entity_name.to_string());
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                gui_state
                    .scene_manager
                    .as_mut()
                    .unwrap()
                    .get_scene_mut(*scene_id)
                    .unwrap()
                    .delete_entity(*entity_id);
                ui.close_menu();
            }
        });
    }
}
