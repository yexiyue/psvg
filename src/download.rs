use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;

use crate::Dialogue;
static FILL_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"fill="\#.*?""#).unwrap());
static STROKE_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"stroke=".*?""#).unwrap());
static WIDTH_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"width="48""#).unwrap());
static HEIGHT_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"height="48""#).unwrap());
static EVENODD: Lazy<regex::Regex> =
    Lazy::new(|| Regex::new(r#"fill-rule="evenodd""#).unwrap());

pub fn read_json(path: &str) -> anyhow::Result<serde_json::Value> {
    let file = fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub async fn run(dialogue: &Dialogue) -> anyhow::Result<()> {
    let data = read_json(&dialogue.file)?;
    let download_dir = std::env::current_dir()?.join(&dialogue.dir);
    tokio::fs::create_dir_all(&download_dir).await?;
    for item in data.as_array().unwrap() {
        let download_dir = download_dir.clone();
        let path = item["path"].as_str().unwrap().to_string();
        let res = tokio::spawn(async move {
            let svg_url = format!(
                "https://lf-scm-us.larksuitecdn.com/whiteboard_icon_source/{}",
                path
            );
            let file_name = path.split("/").last().unwrap();
            let file_path = download_dir.join(file_name);
            let svg = reqwest::get(&svg_url).await?.text().await?;
            let svg = change_svg_color(&svg);
            tokio::fs::write(file_path, svg).await?;
            tracing::info!("download {} success", path);
            Ok::<(), anyhow::Error>(())
        });
        res.await??;
    }
    Ok(())
}

fn change_svg_color(text: &str) -> String {
    let mut replaced = false;
    let evenodd_match = EVENODD.find(text);

    let text1 = FILL_REGEX.replace_all(text, "fill=\"none\"");
    let text2 = FILL_REGEX.replace_all(&text1, |caps: &regex::Captures| {
        if caps.get(0).unwrap().as_str() != "fill=\"none\"" && evenodd_match.is_some() {
            "fill=\"currentColor\"".to_string()
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    });

    let text3 = STROKE_REGEX.replace_all(&text2, |caps: &regex::Captures| {
        replaced = true;
        "stroke=\"currentColor\"".to_string()
    });

    let final_text = if !replaced {
        FILL_REGEX.replace_all(&text3, "fill=\"currentColor\"")
    } else {
        text3.clone()
    };

    let final_text = WIDTH_REGEX.replace_all(&final_text, "width=\"100%\"");
    let final_text = HEIGHT_REGEX.replace_all(&final_text, "height=\"100%\"");

    final_text.into()
}
