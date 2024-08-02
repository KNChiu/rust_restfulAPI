use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::Mutex;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use chrono::prelude::*;

// 定義資料模型的結構
#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct Item {
    id: usize,      // 項目的唯一識別 ID
    name: String,   // 項目的名稱
}

#[derive(Serialize)]
struct Info {
    time: String,
}

// 定義應用程式狀態，包含一個 Mutex 保護的 Vec<Item>
struct AppState {
    items: Mutex<Vec<Item>>,
}

// 負責從 JSON 文件讀取項目
fn load_items() -> Vec<Item> {
    let path = Path::new("items.json");
    if !path.exists() {
        return vec![]; // 如果文件不存在，返回空向量
    }

    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    serde_json::from_str(&contents).unwrap_or_else(|_| vec![]) // 將 JSON 解析為 Vec<Item>
}

// 負責將項目寫入 JSON 文件
fn save_items(items: &Vec<Item>) -> io::Result<()> {
    let path = Path::new("items.json");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true) // 每次寫入時先清空文件
        .open(path)?;
    let data = serde_json::to_string(items).expect("Unable to serialize data");
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// 創建新項目（POST 請求）
#[utoipa::path(
    post,
    path = "/items",
    request_body = Item,
    responses(
        (status = 201, description = "Created new item successfully", body = Item),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/items")]
async fn create_item(item: web::Json<Item>, data: web::Data<AppState>) -> impl Responder {
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖
    items.push(item.into_inner()); // 將新項目添加到 Vec 中
    save_items(&items).expect("Unable to save items"); // 寫入 JSON 文件
    HttpResponse::Created().finish() // 返回 201 Created 響應
}

/// 獲取所有項目（GET 請求）
#[utoipa::path(
    get,
    path = "/items",
    responses(
        (status = 200, description = "Retrieved all items successfully", body = [Item]),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/items")]
async fn get_items(data: web::Data<AppState>) -> impl Responder {
    let items = data.items.lock().unwrap(); // 獲取資料鎖
    web::Json(items.clone()) // 返回所有項目作為 JSON
}

/// 獲取時間（GET 請求）
#[utoipa::path(
    get,
    path = "/system_info",
    responses(
        (status = 200, description = "Retrieved system info successfully", body = [Info]),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/system_info")]
async fn get_system_info(_data: web::Data<AppState>) -> impl Responder {
    let local: DateTime<Local> = Local::now();
    web::Json(Info {
        time: local.to_string(),
    })
}

/// 更新項目（PUT 請求）
#[utoipa::path(
    put,
    path = "/items/{id}",
    params(
        ("id" = usize, Path, description = "ID of the item to update")
    ),
    request_body = Item,
    responses(
        (status = 200, description = "Updated item successfully", body = Item),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[put("/items/{id}")]
async fn update_item(id: web::Path<usize>, item: web::Json<Item>, data: web::Data<AppState>) -> impl Responder {
    let id = id.into_inner(); // 提取 id
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖

    if let Some(existing_item) = items.iter_mut().find(|i| i.id == id) { // 查找存在的項目
        existing_item.name = item.name.clone(); // 更新項目名稱
        save_items(&items).expect("Unable to save items"); // 寫入 JSON 文件
        return HttpResponse::Ok().finish(); // 返回 200 OK 響應
    }
    HttpResponse::NotFound().finish() // 返回 404 Not Found 響應
}

/// 刪除項目（DELETE 請求）
#[utoipa::path(
    delete,
    path = "/items/{id}",
    params(
        ("id" = usize, Path, description = "ID of the item to delete")
    ),
    responses(
        (status = 200, description = "Deleted item successfully"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[delete("/items/{id}")]
async fn delete_item(id: web::Path<usize>, data: web::Data<AppState>) -> impl Responder {
    let id = id.into_inner(); // 提取 id
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖

    if items.iter().any(|i| i.id == id) { // 檢查項目是否存在
        items.retain(|i| i.id != id); // 刪除項目
        save_items(&items).expect("Unable to save items"); // 寫入 JSON 文件
        return HttpResponse::Ok().finish(); // 返回 200 OK 響應
    }
    HttpResponse::NotFound().finish() // 返回 404 Not Found 響應
}

// 定義 OpenAPI 文檔
#[derive(OpenApi)]
#[openapi(
    paths(get_system_info, create_item, get_items, update_item, delete_item),
    components(schemas(Item))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let items = load_items(); // 從 JSON 文件加載項目
    let app_state = web::Data::new(AppState {
        items: Mutex::new(items), // 初始化應用程序狀態
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // 將應用程式狀態傳遞給應用
            .service(get_system_info) // 註冊創建項目的服務
            .service(create_item) // 註冊創建項目的服務
            .service(get_items) // 註冊獲取所有項目的服務
            .service(update_item) // 註冊更新項目的服務
            .service(delete_item) // 註冊刪除項目的服務
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind("127.0.0.1:8080")? // 綁定到指定的 IP 和端口
    .run() // 啟動伺服器
    .await
}
