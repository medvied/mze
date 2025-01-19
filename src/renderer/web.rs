use std::error;

use actix_web::{
    body::EitherBody, middleware, web, App, Either, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use futures_util::StreamExt as _;

use tokio;

use crate::{
    renderer::{EntityPath, UriSearchQuery},
    Container, ContainerTransaction, Record, Renderer, SearchQuery,
    SearchResult, SearchResultAttribute, SearchResultLink, SearchResultRecord,
    SearchResultTag,
};

mod files;
use files::SEARCH_HTML;

pub struct RendererWebState {
    container: Box<dyn Container + Send>,
}

pub struct RendererWeb {
    uri: String,
    state: Option<RendererWebState>,
}

pub struct SearchResultRendererWeb {}
pub struct SearchQueryRendererWeb {}

#[derive(Debug, serde::Serialize)]
struct JsonSearchResult {
    search_interpretation: String,
    search_result: String,
    search_stats: String,
}

impl Renderer for RendererWeb {
    fn new(
        uri: &str,
        container: Box<dyn Container + Send>,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        Ok(Self {
            uri: uri.to_string(),
            state: Some(RendererWebState { container }),
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
        let state = self.state.take().unwrap();
        self.run_async(self.uri.clone(), state)?;
        Ok(())
    }
}

impl RendererWeb {
    #[tokio::main(flavor = "current_thread")]
    async fn run_async(
        &mut self,
        uri: String,
        state: RendererWebState,
    ) -> Result<(), std::io::Error> {
        let data_state = web::Data::new(std::sync::Mutex::new(state));
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(data_state.clone())
                .route("/test", web::get().to(Self::test))
                .route("/", web::get().to(Self::search_html))
                .route("/search", web::get().to(Self::search))
                .service(
                    web::scope("/record")
                        .route("/data", web::get().to(Self::record_data_get))
                        .route("/data", web::put().to(Self::record_data_put))
                        .route("/tags", web::get().to(Self::record_tags_get))
                        .route("/attributes", web::get().to(Self::record_attributes_get))
                        .route("all", web::get().to(Self::record_all_get)),
                )
        })
        .bind(uri)?
        .workers(4)
        .run()
        .await
    }

    async fn test() -> impl Responder {
        HttpResponse::Ok().body("my test")
    }

    async fn search_html() -> impl Responder {
        HttpResponse::Ok().body(SEARCH_HTML)
    }

    async fn search(
        uri_search_query: web::Query<UriSearchQuery>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<impl Responder, Box<dyn std::error::Error>> {
        let mut state = state_data.lock().unwrap();
        let tx: &(dyn ContainerTransaction + '_) =
            &*state.container.begin_transaction()?;
        let search_query = SearchQuery::new(&uri_search_query.q);
        let container_search_result = tx.search(&search_query)?;
        let search_interpretation =
            SearchQueryRendererWeb::render(tx, &search_query);
        let search_result = container_search_result
            .iter()
            .map(|search_result| {
                SearchResultRendererWeb::render(tx, search_result)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let search_stats = String::new();
        Ok(HttpResponse::Ok().json(JsonSearchResult {
            search_interpretation,
            search_result,
            search_stats,
        }))
    }

    async fn record_get(
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<Result<Record, HttpResponse>, Box<dyn std::error::Error>> {
        let mut state = state_data.lock().unwrap();
        let tx = state.container.begin_transaction()?;
        let eid = entity_path.get_id();
        let record = tx.record_get(&eid)?;
        if record.is_none() {
            return Ok(Err(HttpResponse::NotFound().body(format!(
                "Record not found: entity_path={entity_path:?}"
            ))));
        }
        Ok(Ok(record.unwrap()))
    }

    async fn record_data_get(
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<impl Responder, Box<dyn std::error::Error>> {
        let result = Self::record_get(entity_path, state_data).await?;
        Ok(match result {
            Ok(record) => {
                let data = record.data.unwrap_or_default();
                HttpResponse::Ok().body(data)
            }
            Err(response) => response,
        })
    }

    async fn record_tags_get(
        req: HttpRequest,
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<
        Either<HttpResponse<EitherBody<String>>, impl Responder>,
        Box<dyn std::error::Error>,
    > {
        let result = Self::record_get(entity_path, state_data).await?;
        Ok(match result {
            Ok(record) => {
                Either::Left(web::Json(&record.ta.tags).respond_to(&req))
            }
            Err(response) => Either::Right(response),
        })
    }

    async fn record_attributes_get(
        req: HttpRequest,
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<
        Either<HttpResponse<EitherBody<String>>, impl Responder>,
        Box<dyn std::error::Error>,
    > {
        let result = Self::record_get(entity_path, state_data).await?;
        Ok(match result {
            Ok(record) => {
                Either::Left(web::Json(&record.ta.attributes).respond_to(&req))
            }
            Err(response) => Either::Right(response),
        })
    }

    async fn record_data_put(
        mut body: web::Payload,
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<impl Responder, Box<dyn std::error::Error>> {
        let mut bytes = web::BytesMut::new();
        // TODO unlimited memory read from the user here
        while let Some(item) = body.next().await {
            bytes.extend_from_slice(&item?)
        }
        let record = Record {
            ta: Default::default(),
            data: Some(bytes.to_vec()), // TODO memory copy here
        };
        let eid = entity_path.get_id();

        let mut state = state_data.lock().unwrap();
        let mut tx = state.container.begin_transaction()?;
        let eid1 = tx.record_put(&Some(eid), &record)?;
        assert_eq!(eid1, eid); // TODO rewrite to look better
        tx.commit()?;
        Ok(web::Json(eid))
    }

    async fn record_all_get(
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<impl Responder, Box<dyn std::error::Error>> {
        let mut state = state_data.lock().unwrap();
        let tx = state.container.begin_transaction()?;
        let all_records = tx.record_get_all_ids()?;
        Ok(web::Json(all_records))
    }
}

impl SearchResultRendererWeb {
    fn render(
        tx: &(dyn ContainerTransaction + '_),
        search_result: &SearchResult,
    ) -> String {
        match search_result {
            SearchResult::Record(search_result_record) => {
                Self::render_record(tx, search_result_record)
            }
            SearchResult::Link(search_result_link) => {
                Self::render_link(tx, search_result_link)
            }
            SearchResult::Tag(search_result_tag) => {
                Self::render_tag(tx, search_result_tag)
            }
            SearchResult::Attribute(search_result_attribute) => {
                Self::render_attribute(tx, search_result_attribute)
            }
        }
    }

    fn render_record(
        _tx: &(dyn ContainerTransaction + '_),
        _search_result_record: &SearchResultRecord,
    ) -> String {
        String::from("record")
    }

    fn render_link(
        _tx: &(dyn ContainerTransaction + '_),
        _search_result_record: &SearchResultLink,
    ) -> String {
        String::from("link")
    }

    fn render_tag(
        _tx: &(dyn ContainerTransaction + '_),
        _search_result_record: &SearchResultTag,
    ) -> String {
        String::from("tag")
    }

    fn render_attribute(
        _tx: &(dyn ContainerTransaction + '_),
        _search_result_record: &SearchResultAttribute,
    ) -> String {
        String::from("attribute")
    }
}

impl SearchQueryRendererWeb {
    fn render(
        _tx: &(dyn ContainerTransaction + '_),
        search_query: &SearchQuery,
    ) -> String {
        format!("{:?}", search_query)
    }
}
