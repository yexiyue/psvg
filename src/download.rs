use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use std::{
    borrow::BorrowMut,
    fs,
    sync::{Arc, Mutex},
};

use crate::Dialogue;
static FILL_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"fill="\#.*?""#).unwrap());
static STROKE_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"stroke=".*?""#).unwrap());
static WIDTH_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"width="48""#).unwrap());
static HEIGHT_REGEX: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"height="48""#).unwrap());
static EVENODD: Lazy<regex::Regex> = Lazy::new(|| Regex::new(r#"fill-rule="evenodd""#).unwrap());

pub fn read_json(path: &str) -> anyhow::Result<serde_json::Value> {
    let file = fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub async fn run(dialogue: &Dialogue) -> anyhow::Result<()> {
    let data = read_json(&dialogue.file)?;
    let download_dir = std::env::current_dir()?.join(&dialogue.dir);
    let download_error_json: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    tokio::fs::create_dir_all(&download_dir).await?;
    for item in data.as_array().unwrap() {
        let download_dir = download_dir.clone();
        let path = item["path"].as_str().unwrap().to_string();
        let download_error_json = download_error_json.clone();
        let res = tokio::spawn(async move {
            let svg_url = format!(
                "https://lf-scm-us.larksuitecdn.com/whiteboard_icon_source/{}",
                path
            );
            let file_name = path.split("/").last().unwrap();
            let file_path = download_dir.join(file_name);
            let svg = reqwest::get(&svg_url).await;
            match svg {
                Ok(svg) => {
                    let svg = svg.text().await?;
                    tokio::fs::write(&file_path, &svg).await?;
                    let svg = change_svg_color(&svg);
                    tokio::fs::write(&file_path, svg).await?;
                    tracing::info!("download {} success", path);
                }
                Err(e) => {
                    tracing::error!("download error:{}", e);
                    let mut download_error_json = download_error_json.lock().unwrap();
                    
                    download_error_json.borrow_mut().push(json!({
                        "path": path,
                        "error": e.to_string()
                    }));
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        res.await??;
    }
    let err_data = download_error_json.lock().unwrap();
    let error_data: Value = serde_json::from_value(json!(*err_data))?;

    let write = fs::File::create(std::env::current_dir()?.join("download_error.json"))?;
    serde_json::to_writer_pretty(write, &error_data)?;
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

    let text3 = STROKE_REGEX.replace_all(&text2, |_: &regex::Captures| {
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
