const std = @import("std");

pub fn main() !void {
    const addr = try std.net.Address.parseIp("127.0.0.1", 8080);
    var server = try addr.listen(.{ .reuse_address = true });
    defer server.deinit();
    
    std.debug.print("Server: http://127.0.0.1:8080\n", .{});
    
    while (true) {
        var conn = server.accept() catch continue;
        
        var buf: [4096]u8 = undefined;
        const n = conn.stream.read(&buf) catch 0;
        std.debug.print("Read {} bytes\n", .{n});
        
        if (n > 0) {
            const resp = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
            conn.stream.writeAll(resp) catch {};
            std.debug.print("Wrote response\n", .{});
        }
        
        conn.stream.close();
        std.debug.print("Connection closed\n", .{});
    }
}
