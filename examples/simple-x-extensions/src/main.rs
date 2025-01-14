
#[derive(utoipa::OpenApi)]
#[openapi(
  info(
    contact(
      name = "me",
    ),
    license(
      name = "a license",
    ),
  ),
  tags(
    ( name = "my tag", )
  ),
  servers(
    ( url = "https://localhost", 
      variables(
        ("username" = (default = "demo", description = "Default username for API")),
        ("port" = (enum_values("8080", "5000", "4545")))
      ),
    ),
  ),
  paths(
    get_openapi,
  ),
  modifiers( &ApiModify ),
)]
struct ApiDoc;

struct ApiModify;
impl utoipa::Modify for ApiModify {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    fn extend(extensions: &mut Option<utoipa::openapi::extensions::Extensions>, text: &str) {
      extensions.get_or_insert(utoipa::openapi::extensions::ExtensionsBuilder::new().build())
      .merge(utoipa::openapi::extensions::ExtensionsBuilder::new()
        .add("x-ext-modify", text)
        .build()
      );
    }

    extend(&mut openapi.extensions, "[Modify] openapi");
    extend(&mut openapi.info.extensions, "[Modify] openapi>info");
    extend(&mut openapi.info.contact.as_mut().unwrap().extensions, "[Modify] openapi>info>contact");
    extend(&mut openapi.info.license.as_mut().unwrap().extensions, "[Modify] openapi>info>license");

    openapi.tags.get_or_insert(Vec::new())
    .iter_mut().for_each(|i| {extend(&mut i.extensions, "[Modify] openapi>tags>item");});

    fn extend_servers(servers: &mut Option<Vec<utoipa::openapi::server::Server>>, text: &str) {
      servers.get_or_insert(Vec::new())
      .iter_mut().for_each(|i| {
        extend(&mut i.extensions, format!("{text}>servers>item").as_str());
        i.variables.get_or_insert(std::collections::BTreeMap::new())
        .iter_mut().for_each(|(_, i)| {extend(&mut i.extensions, format!("{text}>servers>item>variables>item").as_str());});
      });
    }
    extend_servers(&mut openapi.servers, "[Modify] openapi");

    fn extend_parameters(parameters: &mut Option<Vec<utoipa::openapi::path::Parameter>>, text: &str) {
      parameters.get_or_insert(Vec::new())
      .iter_mut().for_each(|i| {
        extend(&mut i.extensions, format!("{text}>Parameters>item").as_str());
      });
    }

    /* Paths */
    extend(&mut openapi.paths.extensions, "[Modify] openapi>paths");
    openapi.paths.paths.iter_mut()
    .for_each(|(_,i)| {
      extend(&mut i.extensions, "[Modify] openapi>Paths>PathItem");
      extend_servers(&mut i.servers, "[Modify] openapi>Paths>PathItem");
      extend_parameters(&mut i.parameters, "[Modify] openapi>Paths>PathItem");


      /* Extend operation */
      fn extend_operation(operation: &mut utoipa::openapi::path::Operation, text: &str) {
        extend(&mut operation.extensions, text);
        extend_servers(&mut operation.servers, text);
        extend_parameters(&mut operation.parameters, text);

        extend(&mut operation.responses.extensions, format!("{text}>Responses").as_str());
        operation.responses.responses.iter_mut()
        .for_each(|(_, i)| {
          if let utoipa::openapi::RefOr::T(i) = i { extend(&mut i.extensions, format!("{text}>Responses>Response").as_str()); }
        });
        if let Some(request_body) = &mut operation.request_body {
          extend(&mut request_body.extensions, format!("{text}>RequestBody").as_str());
          request_body.content.iter_mut()
          .for_each(|(_, i)| { extend(&mut i.extensions, format!("{text}>RequestBody>Content").as_str()); });
        }
      }
      if let Some(o) = &mut i.get     { extend_operation(o, "[Motify] openapi>Paths>PathItem>GetOperation"); }
      if let Some(o) = &mut i.put     { extend_operation(o, "[Motify] openapi>Paths>PathItem>PutOperation"); }
      if let Some(o) = &mut i.post    { extend_operation(o, "[Motify] openapi>Paths>PathItem>PostOperation"); }
      if let Some(o) = &mut i.delete  { extend_operation(o, "[Motify] openapi>Paths>PathItem>DeleteOperation"); }
      if let Some(o) = &mut i.options { extend_operation(o, "[Motify] openapi>Paths>PathItem>OptionsOperation"); }
      if let Some(o) = &mut i.head    { extend_operation(o, "[Motify] openapi>Paths>PathItem>HeadOperation"); }
      if let Some(o) = &mut i.patch   { extend_operation(o, "[Motify] openapi>Paths>PathItem>PatchOperation"); }
      if let Some(o) = &mut i.trace   { extend_operation(o, "[Motify] openapi>Paths>PathItem>TraceOperation"); }
    });
  }
}

#[utoipa::path(
    get,
    path = "/openapi",
    extensions(
      ("x-ext-macro" = json!("[Macro] openapi>Paths>PathItem>Operation")),
    ),
    params(
      ( "my_param" = String, 
        Query, 
        extensions(
          ("x-ext-macro" = json!( "[Macro] openapi>Paths>PathItem>Operation>Parameters>item" ) )
        )
      ),
    ),
    responses(
      ( status = 200, 
        description = "YAML representation of this api in the OpenAPI v3.1.x format", 
        extensions(
          ("x-ext-macro" = json!("[Macro] openapi>Paths>PathItem>Operation>Responses>200")),
        ),
        content(
          ( str = "text/plain", 
            extensions(
              ("x-ext-macro" = json!("[Macro] openapi>Paths>PathItem>Operation>Responses>200>text/plain")),
            )
          )
        ),
      ),
    ),
    request_body(
      description = "Common description",
      extensions(
        ("x-ext-macro" = json!("[Macro] openapi>Paths>PathItem>Operation>RequestBody")),
      ),
      content(
        ("text/xml"),
        ( "application/json", 
          extensions(
            ("x-ext-macro" = json!("[Macro] openapi>Paths>PathItem>Operation>RequestBody>application/json")),
          ),
        ),
      ),
    ),
    security(
      (),
      ("api_key" = []),
      ("key" = [], "key2" = []),
    ),
)]
async fn get_openapi() -> impl actix_web::Responder {
  use utoipa::OpenApi;
  match ApiDoc::openapi().to_yaml() {
    Ok(yaml) => actix_web::HttpResponse::Ok().body(yaml),
    Err(e)   => actix_web::HttpResponse::InternalServerError().body(format!("{e:?}")),
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  actix_web::HttpServer::new(move || {
    actix_web::App::new()
    .route("/openapi", actix_web::web::get().to(get_openapi))
  })
  .bind((std::net::Ipv4Addr::UNSPECIFIED, 8080))?
  .run()
  .await
}
