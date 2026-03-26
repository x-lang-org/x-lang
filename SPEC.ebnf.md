# X 语言 EBNF 语法规范

> 本文档使用 EBNF（Extended Backus-Naur Form）定义 X 语言的形式语法。

---

## EBNF 约定

| 符号 | 含义 |
|------|------|
| `=` | 定义 |
| `,` | 连接 |
| `\|` | 选择 |
| `[ ]` | 可选（0 或 1 次） |
| `{ }` | 重复（0 或多次） |
| `{ }+` | 重复（1 或多次） |
| `( )` | 分组 |
| `"..."` | 终结符（字面量） |
| `'...'` | 终结符（字面量） |
| `(* ... *)` | 注释 |

---

## 1. 词法元素

### 1.1 字符集

```ebnf
(* 基本字符 *)
letter = lowercase | uppercase | unicode_letter ;
lowercase = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m"
          | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" ;
uppercase = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
          | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" ;
unicode_letter = (* 任何 Unicode 字母字符 *) ;
digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
hex_digit = digit | "a" | "b" | "c" | "d" | "e" | "f" | "A" | "B" | "C" | "D" | "E" | "F" ;
octal_digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" ;
binary_digit = "0" | "1" ;

(* 空白字符 *)
whitespace = " " | "\t" | "\n" | "\r" ;
```

### 1.2 标识符

```ebnf
identifier = identifier_start { identifier_continue } ;
identifier_start = letter | "_" | unicode_letter ;
identifier_continue = letter | digit | "_" | unicode_letter ;
```

### 1.3 关键字

```ebnf
keyword = "let" | "mutable" | "const"
        | "function" | "return" | "async" | "await"
        | "if" | "else" | "match" | "for" | "while" | "loop"
        | "type" | "class" | "trait" | "implement" | "enum" | "record" | "effect"
        | "module" | "import" | "export" | "pub"
        | "try" | "throw"
        | "needs" | "given" | "with"
        | "together" | "race" | "atomic" | "retry"
        | "where" | "sort" | "by" | "is" | "can" | "wait"
        | "true" | "false" | "self" | "Self" ;
```

### 1.4 字面量

```ebnf
literal = integer_literal | float_literal | boolean_literal | string_literal | char_literal
        | list_literal | dict_literal | tuple_literal | unit_literal ;

(* 整数字面量 *)
integer_literal = decimal_literal | hex_literal | octal_literal | binary_literal ;
decimal_literal = digit { digit } ;
hex_literal = "0" ("x" | "X") hex_digit { hex_digit } ;
octal_literal = "0" ("o" | "O") octal_digit { octal_digit } ;
binary_literal = "0" ("b" | "B") binary_digit { binary_digit } ;

(* 浮点字面量 *)
float_literal = decimal_literal "." decimal_literal [ exponent ] | decimal_literal exponent ;
exponent = ("e" | "E") ["+" | "-"] decimal_literal ;

(* 布尔字面量 *)
boolean_literal = "true" | "false" ;

(* 字符串字面量 *)
string_literal = `"` { string_content } `"` ;
string_content = string_char | escape_sequence ;
string_char = (* 任何除 " 和 \ 及换行符外的字符 *) ;
escape_sequence = `\` ("n" | "t" | "r" | `\` | `"` | `'` | "0" | unicode_escape) ;
unicode_escape = "u" hex_digit hex_digit hex_digit hex_digit ;

(* 字符字面量 *)
char_literal = `'` (char_char | escape_sequence) `'` ;
char_char = (* 任何除 ' 和 \ 外的单个字符 *) ;

(* 复合字面量 *)
list_literal = "[" [ expression { "," expression } ] "]" ;
dict_literal = "{" [ dict_entry { "," dict_entry } ] "}" ;
dict_entry = identifier ":" expression | string_literal ":" expression ;
tuple_literal = "(" expression { "," expression } [ "," ] ")" ;
unit_literal = "()" ;
```

### 1.5 注释

```ebnf
comment = line_comment | block_comment ;
line_comment = "//" { (* 任何除换行符外的字符 *) } ;
block_comment = "/*" { block_comment_content } "*/" ;
block_comment_content = (* 任何字符 *) | block_comment ; (* 支持嵌套 *)
```

---

## 2. 类型

```ebnf
type = type_annotation ;

type_annotation = type_expr [ type_suffix ] ;
type_suffix = "?" | "!" ;

type_expr = simple_type | compound_type | function_type | type_variable ;

(* 简单类型 *)
simple_type = type_name | primitive_type | type_reference ;

type_name = identifier ;
primitive_type = "Integer" | "Float" | "Boolean" | "String" | "Character" | "Unit" | "Never" ;

type_reference = type_name [ type_arguments ] ;
type_arguments = "<" type { "," type } ">" ;

(* 复合类型 *)
compound_type = tuple_type | list_type | map_type | option_type | result_type ;

tuple_type = "(" [ type { "," type } ] ")" ;
list_type = "List" "<" type ">" ;
map_type = "Map" "<" type "," type ">" ;
option_type = "Option" "<" type ">" ;
result_type = "Result" "<" type "," type ">" ;

(* 函数类型 *)
function_type = "(" [ param_type_list ] ")" "->" type ;

param_type_list = param_type { "," param_type } ;
param_type = type | "self" ;

(* 类型变量 *)
type_variable = identifier ;

(* 代数数据类型 *)
type_definition = enum_definition | record_definition | alias_definition ;

enum_definition = "enum" identifier [ type_parameters ] "{" { enum_variant } "}" ;
enum_variant = identifier | identifier "(" type_list ")" | identifier "{" field_list "}" ;
type_list = type { "," type } ;

record_definition = "record" identifier [ type_parameters ] "{" field_list "}" ;
field_list = field { "," field } ;
field = identifier ":" type [ default_value ] ;
default_value = "=" expression ;

alias_definition = "type" identifier [ type_parameters ] "=" type ;
type_parameters = "<" identifier { "," identifier } ">" ;
```

---

## 3. 表达式

```ebnf
expression = assignment_expr ;

(* 赋值表达式 *)
assignment_expr = or_expr [ assignment_op expression ] ;
assignment_op = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;

(* 逻辑或 *)
or_expr = and_expr { "or" and_expr } ;

(* 逻辑与 *)
and_expr = not_expr { "and" not_expr } ;

(* 逻辑非 *)
not_expr = "not" not_expr | comparison_expr ;

(* 比较表达式 *)
comparison_expr = range_expr [ comparison_op range_expr ] ;
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">=" ;

(* 范围表达式 *)
range_expr = add_expr [ ".." add_expr ] ;

(* 加法表达式 *)
add_expr = mul_expr { ("+" | "-") mul_expr } ;

(* 乘法表达式 *)
mul_expr = unary_expr { ("*" | "/" | "%") unary_expr } ;

(* 一元表达式 *)
unary_expr = ("-" | "+") unary_expr | postfix_expr ;

(* 后缀表达式 *)
postfix_expr = primary_expr { postfix } ;
postfix = field_access | method_call | index_access | call_arguments | "?"

field_access = "." identifier ;
method_call = "." identifier call_arguments ;
index_access = "[" expression "]" ;
call_arguments = "(" [ argument_list ] ")" ;

argument_list = expression { "," expression } | named_argument { "," named_argument } ;
named_argument = identifier ":" expression ;

(* 主表达式 *)
primary_expr = literal
             | identifier
             | lambda_expr
             | if_expr
             | match_expr
             | block_expr
             | return_expr
             | await_expr
             | try_expr
             | throw_expr
             | atomic_expr
             | paren_expr
             | constructor_expr ;

paren_expr = "(" expression ")" ;

(* Lambda 表达式 *)
lambda_expr = lambda_params "->" ( expression | block ) ;
lambda_params = "(" [ lambda_param_list ] ")" | identifier ;
lambda_param_list = lambda_param { "," lambda_param } ;
lambda_param = identifier [ ":" type ] ;

(* if 表达式 *)
if_expr = "if" expression block [ "else" ( if_expr | block ) ] ;

(* match 表达式 *)
match_expr = "match" expression "{" { match_arm } "}" ;
match_arm = pattern [ guard ] "=>" ( expression | block ) [ "," ] ;
guard = "if" expression ;

(* 块表达式 *)
block_expr = block ;
block = "{" { statement } [ expression ] "}" ;

(* return 表达式 *)
return_expr = "return" [ expression ] ;

(* await 表达式 *)
await_expr = "await" expression ;

(* try 表达式 *)
try_expr = "try" block [ "catch" "(" identifier ")" block ] [ "finally" block ] ;

(* throw 表达式 *)
throw_expr = "throw" expression ;

(* atomic 表达式 *)
atomic_expr = "atomic" expression ;

(* 构造表达式 *)
constructor_expr = type_name constructor_init ;
constructor_init = "{" [ field_init_list ] "}" | "(" [ expression_list ] ")" ;
field_init_list = field_init { "," field_init } ;
field_init = identifier [ ":" expression ] ;
expression_list = expression { "," expression } ;
```

---

## 4. 模式匹配

```ebnf
pattern = literal_pattern
        | identifier_pattern
        | wildcard_pattern
        | tuple_pattern
        | list_pattern
        | constructor_pattern
        | range_pattern
        | or_pattern ;

(* 字面量模式 *)
literal_pattern = integer_literal | float_literal | boolean_literal | string_literal | char_literal ;

(* 标识符模式 *)
identifier_pattern = identifier [ "@" pattern ] ;
wildcard_pattern = "_" ;

(* 元组模式 *)
tuple_pattern = "(" [ pattern { "," pattern } ] ")" ;

(* 列表模式 *)
list_pattern = "[" [ pattern_list ] "]" ;
pattern_list = pattern { "," pattern } [ "," spread_pattern ] | spread_pattern ;
spread_pattern = "..." identifier ;

(* 构造器模式 *)
constructor_pattern = type_name "(" [ pattern_list ] ")" | type_name "{" [ field_pattern_list ] "}" ;
field_pattern_list = field_pattern { "," field_pattern } ;
field_pattern = identifier ":" pattern ;

(* 范围模式 *)
range_pattern = literal_pattern ".." literal_pattern ;

(* 或模式 *)
or_pattern = pattern "|" pattern ;
```

---

## 5. 语句

```ebnf
statement = let_statement
          | const_statement
          | expression_statement
          | while_statement
          | for_statement
          | loop_statement
          | break_statement
          | continue_statement
          | effect_statement ;

(* 变量声明 *)
let_statement = "let" [ "mutable" ] identifier [ ":" type ] "=" expression ;

(* 常量声明 *)
const_statement = "const" identifier [ ":" type ] "=" expression ;

(* 表达式语句 *)
expression_statement = expression ;

(* while 语句 *)
while_statement = "while" expression block ;

(* for 语句 *)
for_statement = "for" identifier "in" expression block ;

(* loop 语句 *)
loop_statement = "loop" block ;

(* break 语句 *)
break_statement = "break" [ expression ] ;

(* continue 语句 *)
continue_statement = "continue" ;

(* 效果语句 *)
effect_statement = "needs" effect_call ;
effect_call = identifier "." identifier "(" [ argument_list ] ")" ;
```

---

## 6. 声明

```ebnf
declaration = function_decl
            | class_decl
            | trait_decl
            | implement_decl
            | type_definition
            | module_decl
            | import_decl
            | export_decl ;

(* 函数声明 *)
function_decl = [ "pub" ] "function" identifier [ type_parameters ] "(" [ param_list ] ")"
              [ "->" type ] [ "with" effect_list ] ( block | "=" expression ) ;
param_list = param { "," param } ;
param = identifier ":" type [ default_value ] ;
effect_list = identifier { "," identifier } ;

(* 类声明 *)
class_decl = [ "pub" ] "class" identifier [ type_parameters ] "{" { class_member } "}" ;
class_member = field_decl | method_decl | constructor_decl ;
field_decl = [ "pub" ] identifier ":" type [ default_value ] ;
method_decl = "function" identifier [ type_parameters ] "(" [ param_list ] ")" [ "->" type ] ( block | "=" expression ) ;
constructor_decl = "function" "new" "(" [ param_list ] ")" [ "->" "Self" ] block ;

(* trait 声明 *)
trait_decl = [ "pub" ] "trait" identifier [ type_parameters ] "{" { trait_method } "}" ;
trait_method = "function" identifier "(" [ param_list ] ")" [ "->" type ] ;

(* implement 声明 *)
implement_decl = "implement" type_name "for" type_name "{" { implement_method } "}" ;
implement_method = "function" identifier "(" [ param_list ] ")" [ "->" type ] ( block | "=" expression ) ;

(* 模块声明 *)
module_decl = "module" module_path ;
module_path = identifier { "." identifier } ;

(* 导入声明 *)
import_decl = "import" module_path [ import_list ] ;
import_list = "{" identifier { "," identifier } "}" ;

(* 导出声明 *)
export_decl = "export" declaration ;
```

---

## 7. 程序结构

```ebnf
program = { declaration } ;

compilation_unit = [ module_decl ] { import_decl } { declaration } ;
```

---

## 8. 效果系统

```ebnf
(* 效果定义 *)
effect_decl = "effect" identifier "{" { effect_operation } "}" ;
effect_operation = "function" identifier "(" [ param_list ] ")" "->" type ;

(* 效果处理 *)
effect_handler = "given" identifier "{" { effect_impl } "}" ;
effect_impl = "function" identifier "(" [ param_list ] ")" "->" type block ;

(* 效果约束 *)
effect_constraint = "with" effect_list ;
```

---

## 9. 并发构造

```ebnf
(* async 函数 *)
async_function = "async" "function" identifier "(" [ param_list ] ")" [ "->" type ] ( block | "=" expression ) ;

(* together 表达式 *)
together_expr = "together" block ;

(* race 表达式 *)
race_expr = "race" block ;

(* atomic 块 *)
atomic_block = "atomic" block ;

(* retry 语句 *)
retry_statement = "retry" [ integer_literal ] block ;
```

---

## 10. 运算符优先级

| 优先级 | 运算符 | 结合性 | 描述 |
|--------|--------|--------|------|
| 1 (最高) | `.` `(` `[` `?` | 左 | 成员访问、调用、索引、可选 |
| 2 | `not` `-` (一元) | 右 | 逻辑非、负号 |
| 3 | `*` `/` `%` | 左 | 乘除、取模 |
| 4 | `+` `-` | 左 | 加减 |
| 5 | `..` | 左 | 范围 |
| 6 | `<` `>` `<=` `>=` `==` `!=` | 无 | 比较 |
| 7 | `and` | 左 | 逻辑与 |
| 8 | `or` | 左 | 逻辑或 |
| 9 | `?` `??` | 右 | 错误传播、空合并 |
| 10 | `|>` | 左 | 管道 |
| 11 | `=` `+=` `-=` `*=` `/=` | 右 | 赋值 |

---

## 11. 完整语法摘要

```ebnf
(* X 语言完整 EBNF *)

program = { declaration } ;

declaration = function_decl | class_decl | trait_decl | implement_decl
            | type_definition | module_decl | import_decl | export_decl | effect_decl ;

statement = let_statement | const_statement | expression_statement
          | while_statement | for_statement | loop_statement
          | break_statement | continue_statement | effect_statement ;

expression = assignment_expr ;

type = type_expr [ "?" | "!" ] ;

pattern = literal_pattern | identifier_pattern | wildcard_pattern
        | tuple_pattern | list_pattern | constructor_pattern
        | range_pattern | or_pattern ;

identifier = identifier_start { identifier_continue } ;
literal = integer_literal | float_literal | boolean_literal | string_literal
        | char_literal | list_literal | dict_literal | tuple_literal | unit_literal ;
```

---

## 附录 A: 保留字

以下标识符保留供未来使用：

```ebnf
reserved = "abstract" | "as" | "assert" | "become" | "box"
         | "do" | "dyn" | "final" | "in" | "macro" | "move"
         | "override" | "priv" | "pure" | "ref" | "sealed"
         | "sizeof" | "static" | "super" | "typeof" | "unsafe"
         | "use" | "virtual" | "yield" ;
```

---

## 附录 B: 运算符完整列表

```ebnf
operator = arithmetic_op | comparison_op | logical_op | bitwise_op | other_op ;

arithmetic_op = "+" | "-" | "*" | "/" | "%" ;
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
logical_op = "and" | "or" | "not" ;
bitwise_op = "&" | "|" | "^" | "<<" | ">>" | ">>>" ;
other_op = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "?" | "??" | "|>" | ".." ;
```

---

*最后更新：2026-03-26*
