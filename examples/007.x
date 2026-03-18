interface Flyable {
    function fly()
}

class Bird implements Flyable {
    function fly() {
        println("Flying")
    }
}
let bird = Bird()
bird.fly()
