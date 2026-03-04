# 高级函数和闭包

接下来，我们将探索一些与函数和闭包相关的高级特性：函数指针以及返回闭包。

## 函数指针

我们已经讨论了如何将闭包传递给函数；你也可以将普通函数传递给函数！当你想传递一个已经定义的函数而不是定义一个新的闭包时，这很有用。函数强制转换为类型 `fn`（带有小写的 f），不要与 `Fn` 闭包 trait 混淆。`fn` 被称为函数指针（function pointer）。使用函数指针的语法与使用闭包作为函数参数的语法类似：

```x
function add_one(x: integer) -> integer {
  x + 1
}

function do_twice(f: function(integer) -> integer, arg: integer) -> integer {
  f(arg) + f(arg)
}

function main() {
  let answer = do_twice(add_one, 5)
  println("答案是: {}", answer)
}
```

这会打印出 `答案是: 12`。我们指定 `do_twice` 的参数 `f` 是一个 `fn`，它接受一个 `integer` 类型的参数并返回一个 `integer`。然后我们可以在 `do_twice` 的主体中调用 `f`。在 `main` 中，我们可以将函数名 `add_one` 作为第一个参数传递给 `do_twice`。

与闭包不同，`fn` 是一个类型而不是一个 trait，所以我们直接将 `fn` 指定为参数类型，而不是声明一个泛型类型参数，并将 `Fn` 作为 trait 约束之一。

函数指针实现了所有三个闭包 trait（`Fn`、`FnMut` 和 `FnOnce`），这意味着你总是可以将函数指针作为参数传递给期望闭包的函数。最好编写使用泛型类型和闭包 trait 之一的函数，这样你的函数就可以接受函数或闭包。

一个你只想接受 `fn` 而不接受闭包的例子是与不需要闭包的外部代码接口时：C 函数可以接受函数作为参数，但 C 没有闭包。

让我们看一个使用 `Option` 的 `map` 方法的例子，你可以选择传递闭包或命名函数：

```x
let list_of_numbers = [1, 2, 3]
let list_of_strings: List<String> = list_of_numbers
  .iter()
  .map(integer::to_string)
  .collect()
```

在这里，我们使用 `integer::to_string`，它是我们之前见过的 `to_string` 函数，通过使用完全限定语法。我们也可以在这里使用闭包，如下所示：

```x
let list_of_numbers = [1, 2, 3]
let list_of_strings: List<String> = list_of_numbers
  .iter()
  .map(|i| i.to_string())
  .collect()
```

无论哪种方式都有效，所以选择你喜欢的风格，或者在代码中已经有很多闭包的情况下使用函数指针来避免添加更多闭包语法。

好的，这就是函数指针！让我们继续讨论返回闭包！

## 返回闭包

闭包由 trait 表示，这意味着你不能直接返回闭包。在大多数情况下，当你想返回一个 trait 时，你可以使用实现该 trait 的具体类型代替函数的返回值。但是你不能对闭包这样做，因为它们没有可返回的具体类型；例如，你不允许使用 `Fn` trait 作为返回类型；编译器会抱怨：

```x
// 这不会编译！
function returns_closure() -> Fn(integer) -> integer {
  |x| x + 1
}
```

编译器给我们的错误是：

```
error: the `Fn` trait cannot be made into an object
```

我们在第 17 章中讨论了这个问题！我们需要使用 trait 对象。下面是我们如何重写返回闭包的函数：

```x
function returns_closure() -> Box<Fn(integer) -> integer> {
  Box::new(|x| x + 1)
}
```

这个代码编译得很好！我们使用 trait 对象 `Box<Fn(integer) -> integer>` 作为返回类型。我们创建了一个闭包并将其装箱，然后返回它。

好的，这就是返回闭包！这有点复杂，但非常有用。
