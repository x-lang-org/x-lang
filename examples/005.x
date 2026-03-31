class Rectangle {
    private let width: integer
    private let height: integer

    public new(width: integer, height: integer) {
        this.width = width
        this.height = height
    }

    public function area() -> integer {
        this.width * this.height
    }
}

let rect = Rectangle(77, 88)
println(rect.area())
