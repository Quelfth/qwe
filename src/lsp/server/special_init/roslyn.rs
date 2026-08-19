use std::{env, fs};

use lsp_types::{Url, notification::{DidChangeConfiguration, Notification}};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{log::{log, log_msg}, lsp::Server};

use lsp_types::*;

#[derive(Debug, Serialize, Deserialize)]
struct ProjectOpenParams {
    projects: Vec<Url>,
}

struct ProjectOpen;

impl Notification for ProjectOpen {
    type Params = ProjectOpenParams;

    const METHOD: &'static str = "project/open";
}

#[derive(Debug, Serialize, Deserialize)]
struct SolutionOpenParams {
    solution: Url,
}

struct SolutionOpen;

impl Notification for SolutionOpen {
    type Params = SolutionOpenParams;

    const METHOD: &'static str = "solution/open";
}

impl Server {
    pub async fn init_roslyn(&mut self) {
        let result = self.socket.notify::<DidChangeConfiguration>(DidChangeConfigurationParams { settings: json! {{
            "csharp|background_analysis": {
                "dotnet_analyzer_diagnostics_scope": "fullSolution",
                "dotnet_compiler_diagnostics_scope": "fullSolution",
            },
            "csharp|symbol_search": {
                "dotnet_search_reference_assemblies": true,
            }
        }} });
        if let Err(e) = result {
            log!(e)
        }

        let Ok(dir) = env::current_dir() else {return};
        let Ok(read_dir) = fs::read_dir(dir) else {return};

        let mut projects = Vec::new();

        for entry in read_dir
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_ok_and(|f| f.is_file()))
        {
            let path = entry.path();
            
            let Some(ext) = path.extension() else {continue};
            let Ok(uri) = Url::from_file_path(path.clone()) else {continue};

            if ext == "sln" || ext == "slnx" {
                log_msg!("Loading solution {}", path.to_string_lossy());
                let result = self.socket.notify::<SolutionOpen>(SolutionOpenParams { solution: uri });
                if let Err(e) = result {
                    log!(e)
                }

                log_msg!("Finished roslyn init using Solution");
                return;
            }

            if ext == "csproj" {
                projects.push(uri);
            }
        }

        for project in &projects {
            log_msg!("Loading project {}", project);
        }

        let result = self.socket.notify::<ProjectOpen>(ProjectOpenParams { projects });
        if let Err(e) = result {
            log!(e);
        }

        //tokio::time::sleep(Duration::from_secs(5)).await;

        log_msg!("Finished roslyn init using Projects");
    }
}
