use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use tide::{Context, Response};

use crate::db::schema::*;
use crate::error::Error;
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(ctx: Context<State>) -> Result<Response, Error> {
    let user = ctx.get_author();
    let state = ctx.state();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
        //? Get total number of crates.
        let crate_count = crates::table
            .select(sql::count(crates::id))
            .first::<i64>(conn)?;

        //? Get total number of crate downloads.
        let total_downloads = crates::table
            .select(sql::sum(crates::downloads))
            .first::<Option<BigDecimal>>(conn)?
            .map_or(0, |dec| {
                dec.to_u64()
                    .expect("download count exceeding u64::max_value()")
            });

        //? Get the 10 most downloaded crates.
        let most_downloaded = crates::table
            .select((crates::name, crates::downloads))
            .order_by(crates::downloads.desc())
            .limit(10)
            .load::<(String, u64)>(conn)?;

        //? Get the 10 most recently updated crates.
        let last_updated = crates::table
            .select((crates::name, crates::updated_at))
            .order_by(crates::updated_at.desc())
            .limit(10)
            .load::<(String, chrono::NaiveDateTime)>(conn)?;

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "total_downloads": helpers::humanize_number(total_downloads),
            "crate_count": helpers::humanize_number(crate_count),
            "most_downloaded": most_downloaded.into_iter().map(|(name, downloads)| json!({
                "name": name,
                "downloads": helpers::humanize_number(downloads),
            })).collect::<Vec<_>>(),
            "last_updated": last_updated.into_iter().map(|(name, date)| json!({
                "name": name,
                "updated_at": helpers::humanize_datetime(date),
            })).collect::<Vec<_>>(),
        });
        Ok(utils::response::html(
            engine.render("index", &context).unwrap(),
        ))
    });

    transaction.await
}
