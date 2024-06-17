pub mod events;
pub mod modules;

pub use actix_web;
pub use anyhow;
pub use serde_json;

#[macro_export]
macro_rules! create_events {
    ($typename: ident: $($event: ty),* $(,)?) => {
        struct $typename;

        impl $crate::modules::EventCollection for $typename {
            fn events() -> ::std::vec::Vec<$crate::modules::RawEvent> {
                vec![$(
                    $crate::modules::RawEvent {
                        event_identifier: <$event as $crate::events::event::Event>::ID,
                        event_description: <$event as $crate::events::event::Event>::DESCRIPTION,
                        event_category: <$event as $crate::events::event::Event>::CATEGORY,
                        query_paginated: |pagination, pool, request| {
                            ::std::boxed::Box::pin(async move {
                                let filter: ::std::result::Result<$crate::actix_web::web::Query<<<$event as $crate::events::event::Event>::DatabaseAdapter as $crate::events::event::DatabaseEventAdapter>::Filter>, $crate::actix_web::error::Error> = <$crate::actix_web::web::Query<<<$event as $crate::events::event::Event>::DatabaseAdapter as $crate::events::event::DatabaseEventAdapter>::Filter> as $crate::actix_web::FromRequest>::extract(&request).await; // wtf is this
                                match filter {
                                    ::std::result::Result::Ok(filter) => {
                                        ::std::result::Result::Ok($crate::events::event::DatabaseEventFilter::query_paginated(&filter.0, &pagination, &pool)
                                            .await
                                            .map_err($crate::modules::http_server::AppError::Db)?
                                            .into_iter()
                                            .map(|event| {
                                                $crate::serde_json::to_value(&event)
                                                    .expect("Error serializing event")
                                            })
                                            .collect::<::std::vec::Vec<_>>())
                                }
                                ::std::result::Result::Err(err) =>
                                    ::std::result::Result::Err($crate::modules::http_server::AppError::Actix(err)),
                                }
                            })
                        },
                        realtime_filter_constructor: |filter| {
                            let filter: <$event as $crate::events::event::Event>::RealtimeEventFilter = $crate::serde_json::from_str(filter).expect("Error deserializing filter");
                            ::std::result::Result::Ok(::std::boxed::Box::new(move |event: &$crate::serde_json::Value| {
                                let event: <$event as $crate::events::event::Event>::EventData = $crate::serde_json::from_value(::std::clone::Clone::clone(&event)).expect("Error deserializing event");
                                <<$event as $crate::events::event::Event>::RealtimeEventFilter as $crate::events::event::RealtimeEventFilter>::matches(&filter, &event)
                            }))
                        },
                        insert_into_postgres: |pool, event| {
                            ::std::boxed::Box::pin(async move {
                                let event: <$event as $crate::events::event::Event>::EventData = $crate::serde_json::from_value(event).expect("Error deserializing event");
                                <<$event as $crate::events::event::Event>::DatabaseAdapter as $crate::events::event::DatabaseEventAdapter>::insert(&event, &pool)
                                    .await
                                    .map_err(|err| $crate::anyhow::anyhow!(err))
                                    .and_then(|res| {
                                        if res.rows_affected() == 1 {
                                            ::std::result::Result::Ok(())
                                        } else {
                                            ::std::result::Result::Err($crate::anyhow::anyhow!("Failed to insert event"))
                                        }
                                    })
                            })
                        },
                        event_data_schema: ::schemars::schema_for!(<$event as $crate::events::event::Event>::EventData),
                        db_filter_schema: ::schemars::schema_for!(<<$event as $crate::events::event::Event>::DatabaseAdapter as $crate::events::event::DatabaseEventAdapter>::Filter),
                    }
                ),*]
            }
        }
    };
}
