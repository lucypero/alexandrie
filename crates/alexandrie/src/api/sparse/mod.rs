use tide::{Request, StatusCode};

use alexandrie_index::Indexer;

use crate::utils;
use crate::State;

/// Route to sparsly access index entries for a crate.
pub async fn get(req: Request<State>) -> tide::Result {
    let fst = req.param("fst")?;
    let snd = req.param("snd").ok();
    let krate = req.param("crate")?;

    let maybe_expected = match krate.len() {
        0 => None,
        1 => Some(("1", None)),
        2 => Some(("2", None)),
        3 => Some(("3", Some(&krate[..1]))),
        _ => Some((&krate[..2], Some(&krate[2..4]))),
    };

    let response = match maybe_expected {
        Some(expected) if (fst, snd) == expected => {
            let records = req.state().index.all_records(krate)?;

            let mut output = String::default();
            for record in records {
                let record = json::to_string(&record)?;
                output.push_str(&record);
                output.push('\n');
            }

            let body = tide::Body::from_string(output);
            tide::Response::builder(StatusCode::Ok).body(body).build()
        }
        _ => utils::response::error(StatusCode::NotFound, "crate could not be found"),
    };

    Ok(response)
}

/// Route to sparsly access the index's configuration.
pub async fn get_config(req: Request<State>) -> tide::Result {
    let config = req.state().index.configuration()?;
    Ok(utils::response::json(&config))
}
