use crate::config::resolve_config;
use anyhow::Result;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration},
    plugins::{
        CheckConfigUpdatesMessage, ConfigChange, FormatResult, PluginInfo,
        PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest,
        SyncPluginHandler,
    },
};
use markup_fmt::{
    FormatError, Hints,
    config::{FormatOptions, Quotes, ScriptFormatter},
    detect_language, format_text,
};

mod config;

pub struct MarkupFmtPluginHandler;

impl SyncPluginHandler<FormatOptions> for MarkupFmtPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        let version = env!("CARGO_PKG_VERSION").to_string();
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: version.clone(),
            config_key: "markup".to_string(),
            help_url: "https://github.com/g-plane/markup_fmt".to_string(),
            config_schema_url: format!(
                "https://plugins.dprint.dev/g-plane/markup_fmt/v{version}/schema.json",
            ),
            update_url: Some("https://plugins.dprint.dev/g-plane/markup_fmt/latest.json".into()),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").into()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<FormatOptions> {
        resolve_config(config, global_config)
    }

    fn check_config_updates(&self, _: CheckConfigUpdatesMessage) -> Result<Vec<ConfigChange>> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<FormatOptions>,
        mut format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        // falling back to HTML allows to format files with unknown extensions, such as .svg
        let language = detect_language(request.file_path).unwrap_or(markup_fmt::Language::Html);

        let format_result = format_text(
            std::str::from_utf8(&request.file_bytes)?,
            language,
            request.config,
            |code, hints| {
                let mut file_name = request
                    .file_path
                    .file_name()
                    .expect("missing file name")
                    .to_owned();
                file_name.push("#.");
                file_name.push(hints.ext);
                let additional_config = build_additional_config(hints, request.config);
                format_with_host(SyncHostFormatRequest {
                    file_path: &request.file_path.with_file_name(file_name),
                    file_bytes: code.as_bytes(),
                    range: None,
                    override_config: &additional_config,
                })
                .and_then(|result| match result {
                    Some(code) => String::from_utf8(code)
                        .map(|s| s.into())
                        .map_err(anyhow::Error::from),
                    None => Ok(code.into()),
                })
            },
        );
        match format_result {
            Ok(code) => Ok(Some(code.into_bytes())),
            Err(FormatError::Syntax(err)) => Err(err.into()),
            Err(FormatError::External(errors)) => {
                let msg = errors.into_iter().fold(
                    String::from("failed to format code with external formatter:\n"),
                    |mut msg, error| {
                        msg.push_str(&format!("{error}\n"));
                        msg
                    },
                );
                Err(anyhow::anyhow!(msg))
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
dprint_core::generate_plugin_code!(
    MarkupFmtPluginHandler,
    MarkupFmtPluginHandler,
    FormatOptions
);

#[doc(hidden)]
pub fn build_additional_config(hints: Hints, config: &FormatOptions) -> ConfigKeyMap {
    let mut additional_config = ConfigKeyMap::new();
    additional_config.insert("lineWidth".into(), (hints.print_width as i32).into());
    additional_config.insert("printWidth".into(), (hints.print_width as i32).into());
    additional_config.insert("fileIndentLevel".into(), (hints.indent_level as i32).into());

    if hints.attr {
        match config.language.quotes {
            Quotes::Double => {
                if matches!(
                    config.language.script_formatter,
                    Some(ScriptFormatter::Biome)
                ) {
                    additional_config.insert("quoteStyle".into(), "single".into());
                } else {
                    additional_config.insert("quoteStyle".into(), "alwaysSingle".into());
                }
            }
            Quotes::Single => {
                if matches!(
                    config.language.script_formatter,
                    Some(ScriptFormatter::Biome)
                ) {
                    additional_config.insert("quoteStyle".into(), "double".into());
                } else {
                    additional_config.insert("quoteStyle".into(), "alwaysDouble".into());
                }
            }
        }
        if hints.ext == "css" {
            additional_config.insert("singleLineTopLevelDeclarations".into(), true.into());
        }
    }

    additional_config
}

#[cfg(test)]
mod tests {
    use super::*;
    use markup_fmt::config::VueCustomBlock;

    #[test]
    fn test_resolve_config_vue_custom_block_simple() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("vue.customBlock".into(), "squash".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert!(matches!(
            result.config.language.vue_custom_block.get("any-block"),
            VueCustomBlock::Squash
        ));
    }

    #[test]
    fn test_resolve_config_vue_custom_block_per_block() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("vue.customBlock".into(), "langAttribute".into());
        config.insert("vue.customBlock.i18n".into(), "none".into());
        config.insert("vue.customBlock.docs".into(), "squash".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert!(matches!(
            result.config.language.vue_custom_block.get("i18n"),
            VueCustomBlock::None
        ));
        assert!(matches!(
            result.config.language.vue_custom_block.get("docs"),
            VueCustomBlock::Squash
        ));
        assert!(matches!(
            result.config.language.vue_custom_block.get("unknown"),
            VueCustomBlock::LangAttribute
        ));
    }

    #[test]
    fn test_resolve_config_language_specific_script_indent() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("scriptIndent".into(), false.into());
        config.insert("vue.scriptIndent".into(), true.into());
        config.insert("html.scriptIndent".into(), false.into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert_eq!(result.config.language.script_indent, false);
        assert_eq!(result.config.language.vue_script_indent, Some(true));
        assert_eq!(result.config.language.html_script_indent, Some(false));
    }

    #[test]
    fn test_resolve_config_language_specific_style_indent() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("styleIndent".into(), false.into());
        config.insert("svelte.styleIndent".into(), true.into());
        config.insert("astro.styleIndent".into(), false.into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert_eq!(result.config.language.style_indent, false);
        assert_eq!(result.config.language.svelte_style_indent, Some(true));
        assert_eq!(result.config.language.astro_style_indent, Some(false));
    }

    #[test]
    fn test_resolve_config_component_whitespace_sensitivity() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("whitespaceSensitivity".into(), "css".into());
        config.insert("component.whitespaceSensitivity".into(), "strict".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert!(
            result
                .config
                .language
                .component_whitespace_sensitivity
                .is_some()
        );
    }

    #[test]
    fn test_resolve_config_invalid_vue_custom_block_value() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("vue.customBlock".into(), "invalid-value".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].property_name, "vue.customBlock");
        assert!(
            result.diagnostics[0]
                .message
                .contains("invalid value for config")
        );
    }

    #[test]
    fn test_resolve_config_invalid_per_block_value() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        config.insert("vue.customBlock".into(), "langAttribute".into());
        config.insert("vue.customBlock.i18n".into(), "invalid".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].property_name, "vue.customBlock.i18n");
    }

    #[test]
    fn test_resolve_config_case_variants() {
        let mut handler = MarkupFmtPluginHandler;
        let mut config = ConfigKeyMap::new();
        // Test both "langAttribute" and "lang-attribute" work
        config.insert("vue.customBlock".into(), "langAttribute".into());

        let global_config = GlobalConfiguration::default();
        let result = handler.resolve_config(config, &global_config);

        assert_eq!(result.diagnostics.len(), 0);
        assert!(matches!(
            result.config.language.vue_custom_block.get("any"),
            VueCustomBlock::LangAttribute
        ));
    }
}
