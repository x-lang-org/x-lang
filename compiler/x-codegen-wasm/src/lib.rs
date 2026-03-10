use wasm_bindgen::prelude::*;
use x_codegen::{CodeGenError, CodegenOutput, Target, get_code_generator, CodeGenConfig};
use x_parser::Parser;

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
            },
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
            },
            Err(error) => Err(JsError::new(&error.to_string())),
        }
    }
}

impl XLangCompiler {
    fn compile(&self, code: &str) -> Result<CodegenOutput, CodeGenError> {
        // 1. 语法分析
        let parser = Parser::new();
        let program = parser.parse(code).map_err(|e| CodeGenError::ParseError(e.to_string()))?;
        
        // 2. 类型检查 - 暂时跳过
        // TODO: 实现类型检查
        
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
