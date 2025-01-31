// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use crate::{
  helpers::{resolve_tauri_path, template, Logger},
  VersionMetadata,
};
use anyhow::Context;
use clap::{ArgSettings, Parser};
use handlebars::{to_json, Handlebars};
use heck::{ToKebabCase, ToSnakeCase};
use include_dir::{include_dir, Dir};
use std::{collections::BTreeMap, env::current_dir, fs::remove_dir_all, path::PathBuf};

const BACKEND_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/backend");
const API_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/with-api");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri plugin project")]
pub struct Options {
  /// Name of your Tauri plugin
  #[clap(short = 'n', long = "name")]
  plugin_name: String,
  /// Initializes a Tauri plugin with TypeScript API
  #[clap(long)]
  api: bool,
  /// Initializes a Tauri core plugin (internal usage)
  #[clap(short, long, hide(true))]
  #[clap(setting(ArgSettings::Hidden))]
  tauri: bool,
  /// Set target directory for init
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
  directory: String,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  tauri_path: Option<PathBuf>,
  /// Author name
  #[clap(short, long)]
  author: Option<String>,
}

impl Options {
  fn load(&mut self) {
    if self.author.is_none() {
      self.author.replace(if self.tauri {
        "Tauri Programme within The Commons Conservancy".into()
      } else {
        "You".into()
      });
    }
  }
}

pub fn command(mut options: Options) -> Result<()> {
  options.load();
  let logger = Logger::new("tauri:init:plugin");
  let template_target_path = PathBuf::from(options.directory).join(&format!(
    "tauri-plugin-{}",
    options.plugin_name.to_kebab_case()
  ));
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../../metadata.json"))?;
  if template_target_path.exists() {
    logger.warn(format!(
      "Plugin dir ({:?}) not empty.",
      template_target_path
    ));
  } else {
    let (tauri_dep, tauri_example_dep, tauri_build_dep) =
      if let Some(tauri_path) = options.tauri_path {
        (
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            r#"{{  path = {:?}, features = [ "api-all" ] }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri-build")
          ),
        )
      } else {
        (
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(
            r#"{{ version = "{}", features = [ "api-all" ] }}"#,
            metadata.tauri
          ),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
        )
      };

    let _ = remove_dir_all(&template_target_path);
    let handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    data.insert("plugin_name_original", to_json(&options.plugin_name));
    data.insert("plugin_name", to_json(options.plugin_name.to_kebab_case()));
    data.insert(
      "plugin_name_snake_case",
      to_json(options.plugin_name.to_snake_case()),
    );
    data.insert("tauri_dep", to_json(tauri_dep));
    data.insert("tauri_example_dep", to_json(tauri_example_dep));
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert("author", to_json(options.author));

    if options.tauri {
      data.insert(
        "license_template",
        to_json(
          "// Copyright {20\\d{2}(-20\\d{2})?} Tauri Programme within The Commons Conservancy
             // SPDX-License-Identifier: Apache-2.0
             // SPDX-License-Identifier: MIT\n\n"
            .replace("  ", "")
            .replace(" //", "//"),
        ),
      );
      data.insert(
        "license_header",
        to_json(
          "// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
             // SPDX-License-Identifier: Apache-2.0
             // SPDX-License-Identifier: MIT\n\n"
            .replace("  ", "")
            .replace(" //", "//"),
        ),
      );
    }

    template::render(
      &handlebars,
      &data,
      if options.api {
        &API_PLUGIN_DIR
      } else {
        &BACKEND_PLUGIN_DIR
      },
      &template_target_path,
    )
    .with_context(|| "failed to render Tauri template")?;
  }
  Ok(())
}
