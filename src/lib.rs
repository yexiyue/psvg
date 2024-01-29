use clap::Parser;
use clap::Subcommand;
use dialogue_macro::Asker;
pub mod download;

#[derive(Debug, Parser)]
#[command(author,version,about,long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 选择一个文件执行操作
    Choice,
}

impl Cli {
    pub fn init() -> anyhow::Result<Dialogue> {
        let cli = Self::parse();
        match cli.command {
            Commands::Choice => {
                let options = Dialogue::find_json_files()?;
                let dialogue = Dialogue::asker().file(&options).dir().finish();
                Ok(dialogue)
            }
        }
    }
}

#[derive(Debug, Asker)]
pub struct Dialogue {
    #[select(prompt = "请选择一个json文件")]
    pub file: String,
    #[input(prompt = "请输入保存的目录", default = "downloads")]
    pub dir: String,
}

impl Dialogue {
    fn find_json_files() -> anyhow::Result<Vec<String>> {
        let current_dir = std::env::current_dir()?;
        let entries = walkdir::WalkDir::new(current_dir);
        let files: Vec<_> = entries
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let file_name = e.clone().into_path();
                if file_name.extension() == Some(std::ffi::OsStr::new("json")) {
                    Some(file_name.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }
}
