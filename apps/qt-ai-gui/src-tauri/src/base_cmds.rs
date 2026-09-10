//! Bản mặc định của app (base prompt / rule / glossary) — bọc `qt_ai_core::base::BaseStore`.
use crate::error::{blocking, CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::base::{BaseSource, BaseStore};
use qt_ai_core::story::{CheckRule, GenreNames, GenreSetting, Glossary, StoryGenre};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseKind {
    Prompt,
    Rules,
    Glossary,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseView {
    pub kind: BaseKind,
    pub setting: GenreSetting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<GenreNames>,
    /// "builtin" | "file"
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<CheckRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary: Option<Glossary>,
}

pub fn source_label(source: BaseSource) -> String {
    match source {
        BaseSource::Builtin => "builtin".to_string(),
        BaseSource::File(_) => "file".to_string(),
    }
}

fn genre_of(kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<StoryGenre> {
    match (kind, names) {
        (BaseKind::Prompt, Some(names)) => Ok(StoryGenre { setting, names }),
        (BaseKind::Prompt, None) => Err(CommandError::new("invalid", "Base prompt cần cả bối cảnh và kiểu tên riêng")),
        (_, _) => Ok(StoryGenre { setting, names: names.unwrap_or_default() }),
    }
}

pub fn view(store: &BaseStore, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    let mut out = BaseView { kind, setting, names: None, source: String::new(), text: None, rules: None, glossary: None };
    match kind {
        BaseKind::Prompt => {
            out.names = Some(genre.names);
            out.source = source_label(store.prompt_source(&genre));
            out.text = Some(store.prompt(&genre));
        }
        BaseKind::Rules => {
            out.source = source_label(store.rules_source(setting));
            out.rules = Some(store.rules(setting));
        }
        BaseKind::Glossary => {
            out.source = source_label(store.glossary_source(setting));
            out.glossary = Some(store.glossary(setting));
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasePayload {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<CheckRule>>,
    #[serde(default)]
    pub glossary: Option<Glossary>,
}

pub fn save(
    store: &BaseStore,
    kind: BaseKind,
    setting: GenreSetting,
    names: Option<GenreNames>,
    payload: BasePayload,
) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    match kind {
        BaseKind::Prompt => store.save_prompt(&genre, payload.text.as_deref().unwrap_or(""))?,
        BaseKind::Rules => store.save_rules(setting, &payload.rules.unwrap_or_default())?,
        BaseKind::Glossary => store.save_glossary(setting, &payload.glossary.unwrap_or_default())?,
    }
    view(store, kind, setting, names)
}

pub fn reset(store: &BaseStore, kind: BaseKind, setting: GenreSetting, names: Option<GenreNames>) -> CmdResult<BaseView> {
    let genre = genre_of(kind, setting, names)?;
    match kind {
        BaseKind::Prompt => store.reset_prompt(&genre)?,
        BaseKind::Rules => store.reset_rules(setting)?,
        BaseKind::Glossary => store.reset_glossary(setting)?,
    }
    view(store, kind, setting, names)
}

#[tauri::command]
pub async fn base_get<R: Runtime>(
    app: AppHandle<R>,
    kind: BaseKind,
    setting: GenreSetting,
    names: Option<GenreNames>,
) -> CmdResult<BaseView> {
    blocking(move || view(&app.state::<AppState>().base_store(), kind, setting, names)).await
}

#[tauri::command]
pub async fn base_save<R: Runtime>(
    app: AppHandle<R>,
    kind: BaseKind,
    setting: GenreSetting,
    names: Option<GenreNames>,
    payload: BasePayload,
) -> CmdResult<BaseView> {
    blocking(move || save(&app.state::<AppState>().base_store(), kind, setting, names, payload)).await
}

#[tauri::command]
pub async fn base_reset<R: Runtime>(
    app: AppHandle<R>,
    kind: BaseKind,
    setting: GenreSetting,
    names: Option<GenreNames>,
) -> CmdResult<BaseView> {
    blocking(move || reset(&app.state::<AppState>().base_store(), kind, setting, names)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_save_reset_theo_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = BaseStore::at(dir.path());
        let v = view(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han)).unwrap();
        assert_eq!(v.source, "builtin");
        assert!(v.text.as_deref().unwrap().len() > 5000);
        assert!(view(&store, BaseKind::Prompt, GenreSetting::Modern, None).is_err());

        let payload = BasePayload { text: Some("# x".into()), rules: None, glossary: None };
        let saved = save(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han), payload).unwrap();
        assert_eq!(saved.source, "file");
        assert_eq!(saved.text.as_deref(), Some("# x"));
        assert_eq!(reset(&store, BaseKind::Prompt, GenreSetting::Modern, Some(GenreNames::Han)).unwrap().source, "builtin");

        let rules = vec![CheckRule { pattern: "a".into(), flags: None, message: "b".into() }];
        let payload = BasePayload { text: None, rules: Some(rules.clone()), glossary: None };
        let saved = save(&store, BaseKind::Rules, GenreSetting::Ancient, None, payload).unwrap();
        assert_eq!(saved.rules, Some(rules));
        assert_eq!(saved.source, "file");

        let json = serde_json::to_value(&saved).unwrap();
        assert_eq!(json["kind"], "rules");
        assert!(json.get("names").is_none());
        assert!(json.get("text").is_none());

        let g = view(&store, BaseKind::Glossary, GenreSetting::Mixed, None).unwrap();
        assert_eq!(g.glossary.as_ref().unwrap().len(), 8);
    }
}
