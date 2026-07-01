use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const ASSETS_BASE: &str = "https://live2d.shelter.net.cn/mirror/bestdori-assets";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundleFile {
    bundle_name: String,
    file_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildData {
    model: BundleFile,
    physics: BundleFile,
    #[serde(default)]
    textures: Vec<BundleFile>,
    #[serde(default)]
    motions: Vec<BundleFile>,
    #[serde(default)]
    expressions: Vec<BundleFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildDataEnvelope {
    #[serde(rename = "Base")]
    base: BuildData,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BestdoriDownloadReport {
    pub model_name: String,
    pub model_path: String,
    pub output_dir: String,
    pub file_count: usize,
    pub written_files: Vec<String>,
}

#[derive(Debug, Copy, Clone)]
enum BundleKind {
    Model,
    Physics,
    Texture,
    Motion,
    Expression,
}

struct DownloadFile {
    url: String,
    relative_path: String,
}

pub fn download_bestdori_model(
    model_name: &str,
    target_dir: &str,
    folder_name: Option<&str>,
) -> Result<BestdoriDownloadReport> {
    let model_name = validate_model_name(model_name)?;
    let output_root = Path::new(target_dir);
    let model_dir = output_root.join(safe_segment(folder_name.unwrap_or(model_name)));
    let data_dir = model_dir.join("data");

    fs::create_dir_all(&data_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            data_dir.to_string_lossy()
        )
    })?;

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("Failed to create HTTP client")?;

    let build_data = fetch_build_data(&client, model_name)?;
    let files = collect_download_files(&build_data);
    let mut written_files = Vec::new();

    for file in &files {
        let output_path = model_dir.join(normalize_relative_path(&file.relative_path)?);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = client
            .get(&file.url)
            .send()
            .with_context(|| format!("Failed to request {}", file.url))?
            .error_for_status()
            .with_context(|| format!("Download failed: {}", file.url))?
            .bytes()
            .with_context(|| format!("Failed to read response: {}", file.url))?;
        fs::write(&output_path, bytes.as_ref())
            .with_context(|| format!("Failed to write {}", output_path.to_string_lossy()))?;
        written_files.push(output_path.to_string_lossy().replace('\\', "/"));
    }

    let model_json = create_model_json(&build_data);
    let model_path = model_dir.join("model.json");
    fs::write(
        &model_path,
        serde_json::to_string_pretty(&model_json).context("Failed to serialize model.json")?,
    )
    .with_context(|| format!("Failed to write {}", model_path.to_string_lossy()))?;
    written_files.push(model_path.to_string_lossy().replace('\\', "/"));

    Ok(BestdoriDownloadReport {
        model_name: model_name.to_string(),
        model_path: model_path.to_string_lossy().to_string(),
        output_dir: model_dir.to_string_lossy().to_string(),
        file_count: written_files.len(),
        written_files,
    })
}

fn fetch_build_data(client: &Client, model_name: &str) -> Result<BuildData> {
    let url = format!("{ASSETS_BASE}/jp/live2d/chara/{model_name}_rip/buildData.asset");
    let envelope = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to request {}", url))?
        .error_for_status()
        .with_context(|| format!("Build data not found: {}", url))?
        .json::<BuildDataEnvelope>()
        .with_context(|| format!("Failed to parse buildData.asset: {}", url))?;
    Ok(envelope.base)
}

fn collect_download_files(build_data: &BuildData) -> Vec<DownloadFile> {
    let mut files = vec![
        DownloadFile {
            url: bundle_asset_url(&build_data.model, BundleKind::Model),
            relative_path: "data/model.moc".to_string(),
        },
        DownloadFile {
            url: bundle_asset_url(&build_data.physics, BundleKind::Physics),
            relative_path: "data/physics.json".to_string(),
        },
    ];

    files.extend(build_data.textures.iter().map(|file| DownloadFile {
        url: bundle_asset_url(file, BundleKind::Texture),
        relative_path: format!(
            "data/textures/{}",
            normalize_file_name(&file.file_name, BundleKind::Texture)
        ),
    }));
    files.extend(build_data.motions.iter().map(|file| DownloadFile {
        url: bundle_asset_url(file, BundleKind::Motion),
        relative_path: format!(
            "data/motions/{}",
            normalize_file_name(&file.file_name, BundleKind::Motion)
        ),
    }));
    files.extend(build_data.expressions.iter().map(|file| DownloadFile {
        url: bundle_asset_url(file, BundleKind::Expression),
        relative_path: format!("data/expressions/{}", file.file_name),
    }));

    files
}

fn create_model_json(build_data: &BuildData) -> Value {
    let motions = build_data
        .motions
        .iter()
        .fold(serde_json::Map::new(), |mut acc, motion| {
            let normalized = normalize_file_name(&motion.file_name, BundleKind::Motion);
            let name = Path::new(&normalized)
                .file_name()
                .and_then(|item| item.to_str())
                .unwrap_or("motion")
                .trim_end_matches(".mtn")
                .to_string();
            acc.insert(
                name,
                json!([{ "file": format!("data/motions/{normalized}") }]),
            );
            acc
        });

    json!({
        "version": "Sample 1.0.0",
        "layout": { "center_x": 0, "center_y": 0, "width": 2 },
        "hit_areas_custom": {
            "head_x": [-0.25, 1],
            "head_y": [0.25, 0.2],
            "body_x": [-0.3, 0.2],
            "body_y": [0.3, -1.9]
        },
        "model": "data/model.moc",
        "physics": "data/physics.json",
        "textures": build_data.textures.iter().map(|texture| {
            format!("data/textures/{}", normalize_file_name(&texture.file_name, BundleKind::Texture))
        }).collect::<Vec<_>>(),
        "motions": motions,
        "expressions": build_data.expressions.iter().map(|expression| {
            json!({
                "name": expression.file_name.trim_end_matches(".exp.json"),
                "file": format!("data/expressions/{}", expression.file_name)
            })
        }).collect::<Vec<_>>()
    })
}

fn bundle_asset_url(file: &BundleFile, kind: BundleKind) -> String {
    format!(
        "{ASSETS_BASE}/jp/{}_rip/{}",
        file.bundle_name,
        normalize_file_name(&file.file_name, kind)
    )
}

fn normalize_file_name(file_name: &str, kind: BundleKind) -> String {
    match kind {
        BundleKind::Model | BundleKind::Motion => file_name.trim_end_matches(".bytes").to_string(),
        BundleKind::Texture => {
            if file_name.ends_with(".bytes") {
                file_name.trim_end_matches(".bytes").to_string() + ".png"
            } else if file_name.contains('.') {
                file_name.to_string()
            } else {
                format!("{file_name}.png")
            }
        }
        BundleKind::Physics | BundleKind::Expression => file_name.to_string(),
    }
}

fn validate_model_name(value: &str) -> Result<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("Invalid Bestdori model name: {value}");
    }
    Ok(trimmed)
}

fn safe_segment(value: &str) -> String {
    let mut segment = String::new();
    for ch in value.chars() {
        if matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            segment.push('_');
        } else {
            segment.push(ch);
        }
    }
    let trimmed = segment.trim_matches('_').trim();
    if trimmed.is_empty() {
        "model".to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_relative_path(value: &str) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    for segment in value.replace('\\', "/").split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            anyhow::bail!("Unsafe relative asset path: {value}");
        }
        path.push(safe_segment(segment));
    }
    Ok(path)
}
