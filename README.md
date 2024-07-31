### 創建新項目（POST）：
`curl -X POST "http://127.0.0.1:8080/items" -H "Content-Type: application/json" -d '{"id": 1, "name": "Item 1"}'`

### 獲取所有項目（GET）：
`curl -X GET "http://127.0.0.1:8080/items"`

### 更新項目（PUT）：
`curl -X PUT "http://127.0.0.1:8080/items/1" -H "Content-Type: application/json" -d '{"id": 1, "name": "Updated Item 1"}'`

### 刪除項目（DELETE）：
`curl -X DELETE "http://127.0.0.1:8080/items/1"`