use anyhow::Result;
use clap::Parser;
use quick_xml::Reader;
use std::path::Path;
use tokio::fs::*;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio_stream::StreamExt;
use tracing::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// 解析結果を出力するJSONファイルへのpath
  #[clap(short, long)]
  output: String,
  /// 法令XMLファイル群が置かれている作業ディレクトリへのpath
  #[clap(short, long)]
  work: String,
  /// 法令ファイルのインデックス情報が書かれたJSONファイルへのpath
  #[clap(short, long)]
  index_file: String,
  /// 検索する単語
  #[clap(short, long)]
  search_words: Vec<String>,
}

async fn init_logger() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();

  init_logger().await?;

  info!("[START] get law data: {:?}", &args.index_file);
  let raw_data_lst = listup_law::get_law_from_index(&args.index_file).await?;
  info!("[END] get law data: {:?}", &args.index_file);

  let mut output_file = File::create(&args.output).await?;
  info!("[START] write json file");
  output_file.write_all("[".as_bytes()).await?;

  let mut law_data_stream = tokio_stream::iter(raw_data_lst);

  let mut is_head = true;

  let work_dir_path = Path::new(&args.work);

  while let Some(law_data) = law_data_stream.next().await {
    let file_path = work_dir_path.join(law_data.file);
    info!("[START] work file: {:?}", file_path);
    let mut reader = Reader::from_reader(BufReader::new(File::open(&file_path).await?));
    let chapter_data =
      search_article_with_word::search_xml(&args.search_words, &mut reader).await?;
    if !chapter_data.chapter_data.is_empty() {
      let chapter_data_lst_json_str = serde_json::to_string(&chapter_data)?;
      info!("[END] work file: {:?}", file_path);
      info!("[START] data write: {:?}", file_path);
      if is_head {
        output_file.write_all("\n".as_bytes()).await?;
        is_head = false;
      } else {
        output_file.write_all(",\n".as_bytes()).await?;
      }
      output_file
        .write_all(chapter_data_lst_json_str.as_bytes())
        .await?;
    }
    info!("[END] data write: {:?}", file_path);
  }

  output_file.write_all("\n]".as_bytes()).await?;
  info!("[END write json file");
  output_file.flush().await?;

  Ok(())
}
