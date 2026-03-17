// Test class with inheritance

class Animal {
    public name: String

    // Constructor using 'new' keyword
    new(name: String) {
        this.name = name
    }

    function speak(self: Animal) -> String {
        "Some sound"
    }
}

class Dog extends Animal {
    public breed: String

    new(name: String, breed: String) {
        this.name = name
        this.breed = breed
    }

    function speak(self: Dog) -> String {
        "Woof!"
    }
}

// Create instances
let dog = Dog("Buddy", "Golden Retriever")
print("Dog created")
