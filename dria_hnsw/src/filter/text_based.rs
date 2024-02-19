use probly_search::{Index, QueryResult};
use probly_search::score::bm25;
use probly_search::score::ScoreCalculator;
use std::borrow::Cow;
use serde_json::{json, Value};

pub struct Wiki {
    pub id: usize,
    pub title: String,
    pub text: String
}

fn tokenizer(s: &str) -> Vec<Cow<str>> {
    s.split(' ').map(Cow::from).collect::<Vec<_>>()
}

fn title_extract(d: &Wiki) -> Vec<&str> {
    vec![d.title.as_str()]
}

fn text_extract(d: &Wiki) -> Vec<&str> {
    vec![d.text.as_str()]
}

pub fn create_index_from_docs(index: &mut Index<usize>, query:&str, metadata: Vec<Value>)-> Vec<Value> {

    let mut wikis = Vec::new();
    let mut query_results = Vec::new();

    let mut iter = 0;

    for (_, value) in metadata.iter().enumerate() {


        let text = value["metadata"]["text"].as_str();
        let title = value["metadata"]["title"].as_str();
        let id = value["id"].as_u64().unwrap() as usize;
        let url = value["metadata"]["url"].as_str();

        let t = text.unwrap().to_string();
        let title = title.unwrap().to_string();

        let sentences = t.split(".");
        for sentence in sentences {
            let wiki = Wiki {
                id: iter,
                title: title.clone(),
                text: sentence.to_string()
            };
            wikis.push(wiki);
            query_results.push(json!({"id":id, "title":title.clone(), "text":sentence.to_string(), "url":url.unwrap()}));

            iter += 1;
        }
    }
    for wiki in wikis.iter() {
        index.add_document(
            &[title_extract, text_extract],
            tokenizer,
            wiki.id.clone(),
            &wiki,
        );
    }

    let results = index.query(query, &mut bm25::new(), tokenizer, &vec![1., 1.]);
    let mut results_as_wiki = vec![];
    for res in results.iter(){
        results_as_wiki.push(query_results[res.key].clone());
    }
    return results_as_wiki
}

