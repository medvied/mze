use std::error;

use actix_web::{
    body::EitherBody, middleware, web, App, Either, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use futures_util::StreamExt as _;

use tokio;

use crate::{renderer::EntityPath, Container, Record, Renderer};

mod files;
use files::SEARCH_HTML;

pub struct RendererWebState {
    container: Box<dyn Container + Send>,
}

pub struct RendererWeb {
    uri: String,
    state: Option<RendererWebState>,
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
                        .route("/attrs", web::get().to(Self::record_attrs_get))
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

    async fn search() -> impl Responder {
        HttpResponse::Ok().body("Ok")
    }

    async fn record_get(
        entity_path: web::Query<EntityPath>,
        state_data: web::Data<std::sync::Mutex<RendererWebState>>,
    ) -> Result<Result<Record, HttpResponse>, Box<dyn std::error::Error>> {
        let mut state = state_data.lock().unwrap();
        let tx = state.container.begin_transaction()?;
        let eid = entity_path.get_id();
        let mut eidv = entity_path.get_id_ver();
        if eidv.is_none() {
            eidv = tx.record_get_ver_latest(&eid)?;
        }
        if eidv.is_none() {
            return Ok(Err(HttpResponse::NotFound().body(format!(
                "Latest version not found: entity_path={entity_path:?}"
            ))));
        }
        let eidv = eidv.unwrap();
        let record = tx.record_get(&eidv)?;
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

    async fn record_attrs_get(
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
                Either::Left(web::Json(&record.ta.attrs).respond_to(&req))
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
        let eidv = tx.record_put(&eid, &record)?;
        tx.commit()?;
        Ok(web::Json(eidv))
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
