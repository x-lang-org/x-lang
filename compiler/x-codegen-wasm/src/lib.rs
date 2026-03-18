use wasm_bindgen::prelude::*;
use x_codegen::{get_code_generator, CodeGenConfig, CodeGenError, CodegenOutput, Target};
use x_parser::Parser;
use x_typechecker::type_check;

#[wasm_bindgen]
pub struct XLangCompiler {
    // 编译器实例
}

#[wasm_bindgen]
impl XLangCompiler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            // 初始化编译器
        }
    }

    #[wasm_bindgen]
    pub fn compile_x_to_ts(&self, code: &str) -> Result<String, JsError> {
        // 编译X语言代码到TypeScript
        match self.compile(code) {
            Ok(output) => {
                // 从files中获取生成的代码
                if let Some(file) = output.files.first() {
                    String::from_utf8(file.content.clone())
                        .map_err(|e| JsError::new(&e.to_string()))
                } else {
                    Err(JsError::new("No code generated"))
                }
            }
            Err(error) => Err(JsError::new(&error.to_string())),
        }
    }

    #[wasm_bindgen]
    pub fn compile_x_to_js(&self, code: &str) -> Result<String, JsError> {
        // 编译X语言代码到TypeScript，然后需要转译为JavaScript
        match self.compile(code) {
            Ok(output) => {
                // 从files中获取生成的代码
                if let Some(file) = output.files.first() {
                    String::from_utf8(file.content.clone())
                        .map_err(|e| JsError::new(&e.to_string()))
                } else {
                    Err(JsError::new("No code generated"))
                }
            }
            Err(error) => Err(JsError::new(&error.to_string())),
        }
    }
}

impl XLangCompiler {
    fn compile(&self, code: &str) -> Result<CodegenOutput, CodeGenError> {
        // 1. 语法分析
        let parser = Parser::new();
        let program = parser
            .parse(code)
            .map_err(|e| CodeGenError::ParseError(e.to_string()))?;

        // 2. 类型检查
        type_check(&program)
            .map_err(|e| CodeGenError::TypeCheckError(e.to_string()))?;

        // 3. 代码生成
        let config = CodeGenConfig::default();
        let mut generator = get_code_generator(Target::TypeScript, config)?;
        let output = generator.generate_from_ast(&program)?;

        Ok(output)
    }
}

#[wasm_bindgen]
pub fn compile_x_to_ts(code: &str) -> Result<String, JsError> {
    let compiler = XLangCompiler::new();
    compiler.compile_x_to_ts(code)
}

#[wasm_bindgen]
pub fn compile_x_to_js(code: &str) -> Result<String, JsError> {
    let compiler = XLangCompiler::new();
    compiler.compile_x_to_js(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_new() {
        let _compiler = XLangCompiler::new();
    }

    // wasm-bindgen 函数在非 wasm 目标上无法调用，需要通过内部 compile 方法测试
    // 这个测试仅在 wasm 目标上运行
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_compile_invalid_code_wasm() {
        let compiler = XLangCompiler::new();
        let result = compiler.compile_x_to_ts("invalid x language code!!");
        // 对于无效代码，应该返回错误
        assert!(result.is_err());
    }

    // 非 wasm 目标上的测试：直接测试内部 compile 方法
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_compile_invalid_code_native() {
        let compiler = XLangCompiler::new();
        // 直接测试内部 compile 方法，绕过 wasm-bindgen 层
        let result = compiler.compile("invalid x language code!!");
        // 对于无效代码，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_function_new() {
        let _compiler_fn = compile_x_to_ts;
        let _compiler_js = compile_x_to_js;
    }
}
