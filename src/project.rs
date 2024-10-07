use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};

// Project metadata structure for project.json
#[derive(Serialize, Deserialize, Debug)]
struct ProjectMetadata {
    project_name: String,
    version: String,
}

impl ProjectMetadata {
    // Convert metadata to JSON format
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

// File management system struct
pub struct FileManagement;

impl FileManagement {
    // Function to create a new project at the specified path
    pub fn create_project(project_name: &str, project_path: &str) {
        let base_path = format!("{}/{}", project_path, project_name);

        // Create main project folder
        FileManagement::create_folder(&base_path);

        // Create subfolders
        FileManagement::create_folder(&format!("{}/assets", base_path));
        FileManagement::create_folder(&format!("{}/assets/images", base_path));
        FileManagement::create_folder(&format!("{}/assets/sounds", base_path));
        FileManagement::create_folder(&format!("{}/assets/fonts", base_path));
        FileManagement::create_folder(&format!("{}/assets/videos", base_path));
        FileManagement::create_folder(&format!("{}/entities", base_path));
        FileManagement::create_folder(&format!("{}/scripts", base_path));
        FileManagement::create_folder(&format!("{}/scenes", base_path));

        // Create project.json
        let metadata = ProjectMetadata {
            project_name: project_name.to_string(),
            version: "1.0.0".to_string(),
        };

        FileManagement::create_project_file(&base_path, &metadata);
        println!("Project '{}' created successfully at {}!", project_name, project_path);
    }

    // Helper function to create folders
    fn create_folder(path: &str) {
        if !Path::new(path).exists() {
            fs::create_dir_all(path).expect("Failed to create folder.");
            println!("Created folder: {}", path);
        }
    }

    // Create project.json file
    fn create_project_file(base_path: &str, metadata: &ProjectMetadata) {
        let file_path = format!("{}/project.json", base_path);
        let mut file = File::create(&file_path).expect("Failed to create project.json.");
        file.write_all(metadata.to_json().as_bytes())
            .expect("Failed to write to project.json.");
        println!("Created project.json with metadata.");
    }
}