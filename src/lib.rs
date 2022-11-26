use anyhow::Result;
use encoding_rs::Encoding;
use quick_xml::{encoding, events::Event, Reader};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::{BufReader, AsyncReadExt}};
use tracing::*;


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct LawParagraph {
  /// 法令番号
  pub num: String,
  /// 見出しと章番号
  pub chapter_data: Vec<Chapter>,
}

/// 章・節などを表す
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize, Serialize)]
pub struct Chapter {
  /// 編
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part: Option<usize>,
  /// 章
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chapter: Option<usize>,
  /// 節
  #[serde(skip_serializing_if = "Option::is_none")]
  pub section: Option<usize>,
  /// 款
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subsection: Option<usize>,
  /// 目
  #[serde(skip_serializing_if = "Option::is_none")]
  pub division: Option<usize>,
  /// 条
  pub article: String,
  /// 項
  #[serde(skip_serializing_if = "Option::is_none")]
  pub paragraph: Option<String>,
  /// 号
  #[serde(skip_serializing_if = "Option::is_none")]
  pub item: Option<String>,
  /// イロハなど（深さも記録する）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sub_item: Option<(usize, String)>,
  /// 附則の場合につける
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppl_provision_title: Option<String>,
}

/// 指定された単語が含まれる条があったとき、その条番号等のデータのみを保存する。
/// 後でこのデータをもとに実際の条文を再度取得するのに使いたい。
pub async fn search_xml(
  search_str: &str,
  reader: &mut Reader<BufReader<File>>,
) -> Result<LawParagraph> {
  let utf8 = Encoding::for_label(b"utf-8").unwrap();

  let mut lst = vec![];
  let mut buf = Vec::new();
  let mut chapter_num = Chapter::default();
  let mut law_num = String::new();
  let mut is_law_num_mode = false;

  reader.trim_text(true);
  loop {
    match reader.read_event_into_async(&mut buf).await {
      Ok(Event::Start(tag)) => match tag.name().as_ref() {
        b"LawNum" => is_law_num_mode = true,
        b"Part" => {
          chapter_num = Chapter {
            part: {
              match chapter_num.part {
                Some(n) => Some(n + 1),
                None => Some(1),
              }
            },
            chapter: None,
            section: None,
            subsection: None,
            division: None,
            article: chapter_num.article,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          };
        }
        b"Chapter" => {
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: {
              match chapter_num.chapter {
                Some(n) => Some(n + 1),
                None => Some(1),
              }
            },
            section: None,
            subsection: None,
            division: None,
            article: chapter_num.article,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"Section" => {
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: {
              match chapter_num.section {
                Some(n) => Some(n + 1),
                None => Some(1),
              }
            },
            subsection: None,
            division: None,
            article: chapter_num.article,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"Subsection" => {
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: {
              match chapter_num.subsection {
                Some(n) => Some(n + 1),
                None => Some(1),
              }
            },
            division: None,
            article: chapter_num.article,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"Division" => {
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: {
              match chapter_num.division {
                Some(n) => Some(n + 1),
                None => Some(1),
              }
            },
            article: chapter_num.article,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"Article" => {
          let article_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: article_num_str,
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          };
          info!("law_num: {}", &law_num);
          info!("law_chapter: {:?}", &chapter_num);
        }
        b"Paragraph" => {
          let paragraph_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: Some(paragraph_num_str),
            item: None,
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"Item" => {
          let item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: Some(item_num_str),
            sub_item: None,
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem1" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((1, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem2" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((2, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem3" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((3, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem4" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((4, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem5" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((5, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem6" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((6, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        b"SubItem7" => {
          let sub_item_num_str = tag
            .attributes()
            .find(|res| encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "Num")
            .map(|res| {
              encoding::decode(&res.unwrap().value, utf8)
                .unwrap()
                .to_string()
            })
            .unwrap();
          chapter_num = Chapter {
            part: chapter_num.part,
            chapter: chapter_num.chapter,
            section: chapter_num.section,
            subsection: chapter_num.subsection,
            division: chapter_num.division,
            article: chapter_num.article,
            paragraph: chapter_num.paragraph,
            item: chapter_num.item,
            sub_item: Some((7, sub_item_num_str)),
            suppl_provision_title: chapter_num.suppl_provision_title,
          }
        }
        // 附則
        b"SupplProvision" => {
          chapter_num = Chapter {
            part: None,
            chapter: None,
            section: None,
            subsection: None,
            division: None,
            article: String::new(),
            paragraph: None,
            item: None,
            sub_item: None,
            suppl_provision_title: tag
              .attributes()
              .find(|res| {
                encoding::decode(res.as_ref().unwrap().key.0, utf8).unwrap() == "AmendLawNum"
              })
              .map(|res| {
                encoding::decode(&res.unwrap().value, utf8)
                  .unwrap()
                  .to_string()
              }),
          }
        }
        _ => (),
      },
      Ok(Event::End(tag)) => {
        if let b"LawNum" = tag.name().as_ref() {
          is_law_num_mode = false
        }
      }
      Ok(Event::Text(text)) => {
        if is_law_num_mode {
          law_num = encoding::decode(&text.into_inner(), utf8)?.to_string();
        } else {
          let text_str = encoding::decode(&text.into_inner(), utf8)?.to_string();
          let is_use_junyou = text_str.contains(search_str);
          info!("law_num: {}", &law_num);
          if is_use_junyou {
            lst.push(chapter_num.clone())
          }
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => panic!("法令名APIの結果のXMLの解析中のエラー: {}", e),
      _ => (),
    }
  }
  lst.sort();
  lst.dedup();
  Ok(LawParagraph {
    num: law_num,
    chapter_data: lst,
  })
}

pub async fn get_law_from_artcile_info(info_file_path: &str) -> Result<Vec<LawParagraph>> {
  let mut f = File::open(info_file_path).await?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf).await?;
  let file_str = std::str::from_utf8(&buf)?;
  let raw_data_lst = serde_json::from_str(&file_str)?;
  Ok(raw_data_lst)
}


