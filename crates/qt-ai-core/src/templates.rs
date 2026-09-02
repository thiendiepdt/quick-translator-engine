//! Template AGENTS.md + workflows cho Antigravity — nhúng từ apps/qt-ai-cli/antigravity để hai bản
//! TS/Rust chỉ có một nguồn. Cargo theo dõi file include_str! nên sửa template là rebuild.

use crate::error::{CoreError, Result};
use crate::story_fs::write_text;
use std::fs;
use std::path::Path;

pub const AGENTS_MD: &str = include_str!("../../../apps/qt-ai-cli/antigravity/AGENTS.md");
pub const WORKFLOWS: [(&str, &str); 2] = [
    ("setup-story.md", include_str!("../../../apps/qt-ai-cli/antigravity/workflows/setup-story.md")),
    ("translate.md", include_str!("../../../apps/qt-ai-cli/antigravity/workflows/translate.md")),
];

pub fn render(source: &str, qt_ai_command: &str, root: &str) -> String {
    source.replace("{{QT_AI}}", qt_ai_command).replace("{{STORY_ROOT}}", root)
}

/// Copy template vào folder truyện; KHÔNG ghi đè file đã tồn tại (người dùng có thể đã sửa tay).
pub fn copy_templates(root: &Path, qt_ai_command: &str) -> Result<()> {
    let root_text = root.display().to_string();
    let agents = root.join("AGENTS.md");
    if !agents.exists() {
        write_text(&agents, &render(AGENTS_MD, qt_ai_command, &root_text))?;
    }
    let workflows_dir = root.join(".agent").join("workflows");
    fs::create_dir_all(&workflows_dir).map_err(CoreError::io(&workflows_dir))?;
    for (name, source) in WORKFLOWS {
        let target = workflows_dir.join(name);
        if target.exists() {
            continue;
        }
        write_text(&target, &render(source, qt_ai_command, &root_text))?;
    }
    Ok(())
}
