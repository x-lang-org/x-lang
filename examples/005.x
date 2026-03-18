struct Rectangle {
    let width: integer
    let height: integer

    public new(width: Int64, height: Int64) {
        this.width = width
        this.height = height
    }

    public function area() {
        width * height
    }
}

let rect = Rectangle(width: 77, height: 88)
println(rect.area())
