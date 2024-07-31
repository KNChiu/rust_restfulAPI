use actix_web::{get, post, put, delete, web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// 定義資料模型的結構
#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: usize,      // 項目的唯一識別 ID
    name: String,   // 項目的名稱
}

// 定義應用程式狀態，包含一個 Mutex 保護的 Vec<Item>
struct AppState {
    items: Mutex<Vec<Item>>,
}

// 創建新項目（POST 請求）
#[post("/items")]
async fn create_item(item: web::Json<Item>, data: web::Data<AppState>) -> impl Responder {
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖
    items.push(item.into_inner()); // 將新項目添加到 Vec 中
    HttpResponse::Created().finish() // 返回 201 Created 響應
}

// 獲取所有項目（GET 請求）
#[get("/items")]
async fn get_items(data: web::Data<AppState>) -> impl Responder {
    let items = data.items.lock().unwrap(); // 獲取資料鎖
    web::Json(items.clone()) // 返回所有項目作為 JSON
}

// 更新項目（PUT 請求）
#[put("/items/{id}")]
async fn update_item(id: web::Path<usize>, item: web::Json<Item>, data: web::Data<AppState>) -> impl Responder {
    let id = id.into_inner(); // 提取 id
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖

    if let Some(existing_item) = items.iter_mut().find(|i| i.id == id) { // 查找存在的項目
        existing_item.name = item.name.clone(); // 更新項目名稱
        return HttpResponse::Ok().finish(); // 返回 200 OK 響應
    }
    HttpResponse::NotFound().finish() // 返回 404 Not Found 響應
}

// 刪除項目（DELETE 請求）
#[delete("/items/{id}")]
async fn delete_item(id: web::Path<usize>, data: web::Data<AppState>) -> impl Responder {
    let id = id.into_inner(); // 提取 id
    let mut items = data.items.lock().unwrap(); // 獲取資料鎖

    if items.iter().any(|i| i.id == id) { // 檢查項目是否存在
        items.retain(|i| i.id != id); // 刪除項目
        return HttpResponse::Ok().finish(); // 返回 200 OK 響應
    }
    HttpResponse::NotFound().finish() // 返回 404 Not Found 響應
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        items: Mutex::new(vec![]), // 初始化一個空的項目列表
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // 將應用程式狀態傳遞給應用
            .service(create_item) // 註冊創建項目的服務
            .service(get_items) // 註冊獲取所有項目的服務
            .service(update_item) // 註冊更新項目的服務
            .service(delete_item) // 註冊刪除項目的服務
    })
    .bind("127.0.0.1:8080")? // 綁定到指定的 IP 和端口
    .run() // 啟動伺服器
    .await
}
