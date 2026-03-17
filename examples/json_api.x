// JSON API Example

function user_json(id: integer, name: string, email: string, role: string) -> string {
    let id_str = int_to_string(id)
    let part1 = concat("""{"id": """, concat(id_str, """, "name": """))
    let part2 = concat(name, """, "email": """)
    let part3 = concat(email, """, "role": """)
    let part4 = concat(role, """}""")
    concat(part1, concat(part2, concat(part3, part4)))
}

function get_users() -> string {
    let u1 = user_json(1, "Alice", "alice@example.com", "admin")
    let u2 = user_json(2, "Bob", "bob@example.com", "developer")
    let u3 = user_json(3, "Charlie", "charlie@example.com", "manager")
    concat("""{"users": [""", concat(u1, concat(", ", concat(u2, concat(", ", concat(u3, "]")))))))
}

function main() {
    println("=== JSON API ===")
    println(get_users())
}
