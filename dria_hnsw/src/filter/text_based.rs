use probly_search::score::ScoreCalculator;
use probly_search::score::{bm25, zero_to_one};
use probly_search::Index;
use serde_json::{json, Value};
use std::borrow::Cow;

pub struct Doc {
    pub id: usize,
    pub text: String,
}

fn tokenizer(s: &str) -> Vec<Cow<str>> {
    s.split(' ').map(Cow::from).collect::<Vec<_>>()
}

fn text_extract(d: &Doc) -> Vec<&str> {
    vec![d.text.as_str()]
}

pub fn create_index_from_docs(
    index: &mut Index<usize>,
    query: &str,
    metadata: Vec<Value>,
) -> Vec<Value> {
    let mut wikis = Vec::new();
    let mut query_results = Vec::new();
    let mut ids = Vec::new();
    let mut iter = 0;

    for value in metadata.iter() {
        let mut text = value["metadata"]["text"].as_str();
        if text.is_none() {
            //use whole metadata as text
            text = value["metadata"].as_str();
        }

        let id_doc = value["id"].as_u64().unwrap() as usize;
        let url = value["metadata"]["url"].as_str();

        let t = text.unwrap().to_string();

        let sentences = t.split(".");
        for sentence in sentences {
            let wiki = Doc {
                id: iter,
                text: sentence.to_string(),
            };
            wikis.push(wiki);

            let mut value_x = value.clone();
            value_x["metadata"]["text"] = json!(sentence.to_string());
            query_results.push(value_x);
            ids.push(id_doc);
            iter += 1;
        }
    }
    if wikis.len() == 0 {
        return metadata;
    }

    for wiki in wikis.iter() {
        index.add_document(&[text_extract], tokenizer, wiki.id.clone(), &wiki);
    }

    let results = index.query(query, &mut zero_to_one::new(), tokenizer, &[1.]);
    let mut results_as_wiki = vec![];
    for res in results.iter() {
        let val = json!({"id": ids[res.key], "metadata": query_results[res.key].clone(), "score": res.score});
        results_as_wiki.push(val);
    }
    return results_as_wiki;
}
