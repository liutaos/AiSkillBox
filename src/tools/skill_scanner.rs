use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;
use yaml_rust2::YamlLoader;

/// 扫描到的 skill 信息
#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub path: PathBuf,
    pub skill_file: PathBuf,
}

/// 扫描 skills 目录，解析 SKILL.md 的 front matter
pub fn scan_skills(skills_dir: &str) -> Vec<SkillInfo> {
    let dir = Path::new(skills_dir);
    if !dir.exists() {
        warn!("skills 目录不存在: {}", skills_dir);
        return Vec::new();
    }

    let mut skills = Vec::new();

    for entry in WalkDir::new(skills_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                match parse_skill_file(&skill_file) {
                    Some((name, description, tags)) => {
                        let short_desc = if description.len() > 50 {
                            let mut end = 50;
                            while end > 0 && !description.is_char_boundary(end) {
                                end -= 1;
                            }
                            format!("{}...", &description[..end])
                        } else {
                            description.clone()
                        };
                        info!("发现 skill: {} - {}", name, short_desc);
                        skills.push(SkillInfo {
                            name,
                            description,
                            tags,
                            path: path.to_path_buf(),
                            skill_file,
                        });
                    }
                    None => {
                        let dir_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        warn!("SKILL.md 没有 front matter: {:?}", skill_file);
                        skills.push(SkillInfo {
                            name: dir_name.clone(),
                            description: format!("Skill: {}", dir_name),
                            tags: Vec::new(),
                            path: path.to_path_buf(),
                            skill_file,
                        });
                    }
                }
            }
        }
    }

    info!("共扫描到 {} 个 skill", skills.len());
    skills
}

/// 解析 SKILL.md 的 YAML front matter
/// 返回 (name, description, tags)
fn parse_skill_file(skill_file: &Path) -> Option<(String, String, Vec<String>)> {
    let content = std::fs::read_to_string(skill_file).ok()?;

    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end_pos = after_first.find("---")?;
    let front_matter = &after_first[..end_pos];

    let docs = YamlLoader::load_from_str(front_matter).ok()?;
    let doc = docs.first()?;

    let name = doc["name"].as_str()?.to_string();
    let description = doc["description"].as_str()?.to_string();
    
    let tags: Vec<String> = doc["tags"]
        .as_vec()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Some((name, description, tags))
}

/// 将 SkillInfo 转换为 ToolDefinition 格式的 JSON
pub fn skill_to_tool_json(skill: &SkillInfo) -> serde_json::Value {
    serde_json::json!({
        "name": skill.name,
        "description": skill.description,
        "inputSchema": {
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Skill文件路径",
                    "default": skill.skill_file.to_string_lossy()
                }
            }
        },
        "handler": "file_reader"
    })
}
