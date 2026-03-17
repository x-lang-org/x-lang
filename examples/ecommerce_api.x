// E-Commerce API Demo - Using consistent types for dictionaries

function main() {
    println("=== E-Commerce API Response Demo ===")
    println("")

    // 1. User API - All string values
    println("GET /api/users/1001")
    let _user: Dictionary<string, string> = {
        "id" => "1001",
        "name" => "Zhang San",
        "email" => "zhangsan@example.com",
        "role" => "admin"
    }

    let _addr: Dictionary<string, string> = {
        "id" => "1",
        "label" => "Home",
        "street" => "88 Tech Road",
        "city" => "Shenzhen"
    }

    println("User API response created")
    println("")

    // 2. Products API - All integer values
    println("GET /api/products")
    let _p1: Dictionary<string, integer> = {
        "id" => 101,
        "price" => 15999,
        "stock" => 50
    }

    let _p2: Dictionary<string, integer> = {
        "id" => 102,
        "price" => 9999,
        "stock" => 100
    }

    let _p3: Dictionary<string, integer> = {
        "id" => 103,
        "price" => 1899,
        "stock" => 200
    }

    println("Products API response created")
    println("")

    // 3. Order API - All integer values
    println("GET /api/orders/ORD2024031701")

    let _item1: Dictionary<string, integer> = {
        "id" => 1,
        "product_id" => 101,
        "quantity" => 1,
        "unit_price" => 15999
    }

    let _item2: Dictionary<string, integer> = {
        "id" => 2,
        "product_id" => 103,
        "quantity" => 2,
        "unit_price" => 1899
    }

    let _shipping: Dictionary<string, string> = {
        "recipient" => "Zhang San",
        "phone" => "13800138000",
        "city" => "Shanghai",
        "country" => "China"
    }

    let _payment: Dictionary<string, string> = {
        "method" => "alipay",
        "status" => "paid"
    }

    println("Order API response created")
    println("")

    // 4. Dashboard API - All integer values
    println("GET /api/dashboard/stats")
    let _stats: Dictionary<string, integer> = {
        "total_users" => 15823,
        "active_users_today" => 3421,
        "total_orders" => 89234,
        "pending_orders" => 89
    }

    println("Dashboard API response created")
    println("")

    // 5. Cart API - All integer values
    println("GET /api/cart")
    let _cart_item: Dictionary<string, integer> = {
        "id" => 1,
        "product_id" => 101,
        "quantity" => 1,
        "price" => 15999
    }

    let _cart_summary: Dictionary<string, integer> = {
        "total_items" => 1,
        "subtotal" => 15999,
        "total" => 15999
    }

    println("Cart API response created")
    println("")

    println("=== API Demo Complete ===")
}
