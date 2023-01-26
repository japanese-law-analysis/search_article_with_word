[![Workflow Status](https://github.com/japanese-law-analysis/search_article_with_word/workflows/Rust%20CI/badge.svg)](https://github.com/japanese-law-analysis/search_article_with_word/actions?query=workflow%3A%22Rust%2BCI%22)

# search_article_with_word

法律のXMLファイルがあるフォルダから、指定する単語が含まれている条項を探すソフトウェア


## Install

```sh
cargo install --git "https://github.com/japanese-law-analysis/search_article_with_word.git"
```

## Use

```sh
search_article_with_word --output output.json --work "path/to/law_xml_directory" --index-file "path/to/law_list.json" --search-word "word1" --search-word "word2"
```

で起動します。それぞれのオプションの意味は以下の通りです。

- `--output`：指定した単語が含まれる条項の情報のリストを出力するJSONファイル名
- `--work`：[e-gov法令検索](https://elaws.e-gov.go.jp/)からダウンロードした全ファイルが入っているフォルダへのpath
- `--index-file`：[japanese-law-analysis/listup_law](https://github.com/japanese-law-analysis/listup_law)で生成した法令のリストが書かれているJSONファイルへのpath
- `--search-word`：検索する単語を指定する。複数指定可


License: MIT
