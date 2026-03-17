const std = @import("std");

pub fn main() !void {
    const addr = try std.net.Address.parseIp("127.0.0.1", 8080);
    var server = try addr.listen(.{
        .reuse_address = true,
    });
    defer server.deinit();

    std.debug.print("HTTP Server listening on http://127.0.0.1:8080\n", .{});
    std.debug.print("Endpoints:\n", .{});
    std.debug.print("  GET /              - Hello\n", .{});
    std.debug.print("  GET /api/user      - User JSON\n", .{});
    std.debug.print("  GET /api/products  - Products JSON\n", .{});
    std.debug.print("  GET /api/order     - Order JSON\n", .{});
    std.debug.print("  GET /api/dashboard - Dashboard JSON\n", .{});
    std.debug.print("\nPress Ctrl+C to stop\n\n", .{});

    while (true) {
        var conn = server.accept() catch continue;
        defer conn.stream.close();

        handleConnection(&conn) catch {};
    }
}

fn handleConnection(conn: *std.net.Server.Connection) !void {
    var buf: [8192]u8 = undefined;

    // Read request with timeout
    var total_read: usize = 0;
    while (total_read < buf.len) {
        const n = conn.stream.read(buf[total_read..]) catch |err| {
            if (err == error.WouldBlock or err == error.NotOpenForReading) break;
            return err;
        };
        if (n == 0) break;
        total_read += n;
        // Check if we've read the end of headers
        if (std.mem.indexOf(u8, buf[0..total_read], "\r\n\r\n")) |_| {
            break;
        }
    }

    if (total_read == 0) return;

    const request = buf[0..total_read];

    // Parse request line
    var lines = std.mem.splitSequence(u8, request, "\r\n");
    const request_line = lines.next() orelse return;

    var parts = std.mem.splitScalar(u8, request_line, ' ');
    _ = parts.next(); // method
    const path = parts.next() orelse "/";

    // Route handling
    const response: []const u8 = if (std.mem.startsWith(u8, path, "/api/user"))
        userJson()
    else if (std.mem.startsWith(u8, path, "/api/products"))
        productsJson()
    else if (std.mem.startsWith(u8, path, "/api/order"))
        orderJson()
    else if (std.mem.startsWith(u8, path, "/api/dashboard"))
        dashboardJson()
    else if (std.mem.startsWith(u8, path, "/"))
        helloJson()
    else
        "{\"error\": \"Not Found\"}";

    // Build and send response
    const header = std.fmt.bufPrint(buf[0..],
        \\HTTP/1.1 200 OK
        \\Content-Type: application/json; charset=utf-8
        \\Access-Control-Allow-Origin: *
        \\Content-Length: {}
        \\
        \\
    , .{response.len}) catch return;

    // Write header
    var written: usize = 0;
    while (written < header.len) {
        written += conn.stream.write(header[written..]) catch return;
    }

    // Write body
    written = 0;
    while (written < response.len) {
        written += conn.stream.write(response[written..]) catch return;
    }
}

fn helloJson() []const u8 {
    return
        \\{
        \\  "success": true,
        \\  "message": "Welcome to E-Commerce API",
        \\  "version": "1.0.0",
        \\  "endpoints": ["/api/user", "/api/products", "/api/order", "/api/dashboard"]
        \\}
    ;
}

fn userJson() []const u8 {
    return
        \\{
        \\  "success": true,
        \\  "message": "User retrieved successfully",
        \\  "data": {
        \\    "user": {
        \\      "id": 1001,
        \\      "name": "Zhang San",
        \\      "email": "zhangsan@example.com",
        \\      "role": "admin",
        \\      "vip_level": 5,
        \\      "active": true
        \\    },
        \\    "addresses": [
        \\      {"id": 1, "label": "Home", "city": "Shenzhen", "country": "China"},
        \\      {"id": 2, "label": "Office", "city": "Beijing", "country": "China"}
        \\    ]
        \\  }
        \\}
    ;
}

fn productsJson() []const u8 {
    return
        \\{
        \\  "success": true,
        \\  "message": "Products retrieved successfully",
        \\  "data": {
        \\    "products": [
        \\      {"id": 101, "name": "MacBook Pro 14", "price": 15999.0, "stock": 50},
        \\      {"id": 102, "name": "iPhone 15 Pro", "price": 9999.0, "stock": 100},
        \\      {"id": 103, "name": "AirPods Pro 2", "price": 1899.0, "stock": 200}
        \\    ],
        \\    "pagination": {"page": 1, "total": 128}
        \\  }
        \\}
    ;
}

fn orderJson() []const u8 {
    return
        \\{
        \\  "success": true,
        \\  "message": "Order retrieved successfully",
        \\  "data": {
        \\    "order_id": 2024031701,
        \\    "order_no": "ORD2024031701",
        \\    "status": "shipped",
        \\    "items": [
        \\      {"id": 1, "product_name": "MacBook Pro 14", "quantity": 1, "unit_price": 15999.0},
        \\      {"id": 2, "product_name": "AirPods Pro 2", "quantity": 2, "unit_price": 1899.0}
        \\    ],
        \\    "total": 18397.0
        \\  }
        \\}
    ;
}

fn dashboardJson() []const u8 {
    return
        \\{
        \\  "success": true,
        \\  "message": "Dashboard stats retrieved successfully",
        \\  "data": {
        \\    "total_users": 15823,
        \\    "active_users_today": 3421,
        \\    "total_orders": 89234,
        \\    "total_revenue": 2847592.50
        \\  }
        \\}
    ;
}
